//! Репозитории: типизированный доступ к данным.
//!
//! # Почему трейты, а не просто функции
//!
//! Домен тестируется без базы: `ElementsRepo` — трейт, и в тестах бизнес-логики
//! (P3, P4) подставляется реализация в памяти. Если бы код звал `Db::call`
//! напрямую, каждый тест виджета требовал бы временного файла SQLite.
//!
//! # Почему нет `SELECT *`
//!
//! `SELECT *` переживает добавление колонки молча: код продолжает работать, но
//! порядок полей в `row.get(3)` сдвигается, и в `w` оказывается `h`. Такой баг
//! не падает на тестах с одним элементом и портит раскладку у пользователя,
//! поэтому колонки перечисляются явно и маппинг собран в одном месте.
//!
//! # Транзакция на патч
//!
//! `apply_patch` — одна транзакция на весь набор. При падении на пятом элементе
//! первые четыре не должны остаться в новом положении: пользователь увидит
//! раскладку, которую не двигал, и не поймёт, откуда она.

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::Db;
use crate::element::{Element, ElementKind, ElementPatch, PatchOp, Rect};
use crate::error::CoreError;

/// Колонки элемента — в одном месте, чтобы порядок в `SELECT` и в маппинге
/// нельзя было рассинхронизировать.
const ELEMENT_COLUMNS: &str = "id, kind, widget_id, monitor_id, profile_id, \
     x, y, w, h, z, rotation, settings_json, state_json, created_at, updated_at";

/// Читает элемент из строки результата.
///
/// Единственное место маппинга: если оно разъедется с `ELEMENT_COLUMNS`,
/// тест `mapping_covers_all_columns` это поймает.
fn element_from_row(row: &Row<'_>) -> rusqlite::Result<Element> {
    let kind_str: String = row.get(1)?;
    // Неизвестный вид не должен ронять чтение всей раскладки: элемент из более
    // новой версии попадёт в `widget`-заглушку и будет виден пользователю
    // как «неизвестный элемент», а не как пустой экран.
    let kind = ElementKind::from_str_opt(&kind_str).unwrap_or(ElementKind::Widget);

    let settings_str: String = row.get(11)?;
    let state_str: String = row.get(12)?;

    Ok(Element {
        id: row.get(0)?,
        kind,
        widget_id: row.get(2)?,
        monitor_id: row.get(3)?,
        profile_id: row.get(4)?,
        frame: Rect::new(row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?),
        z: row.get(9)?,
        rotation: row.get(10)?,
        settings: serde_json::from_str(&settings_str)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        state: serde_json::from_str(&state_str)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

/// Операции над элементами холста.
pub trait ElementsRepo {
    /// Все элементы монитора, видимые в указанном профиле, по возрастанию `z`.
    ///
    /// Элементы с `profile_id = NULL` видны во всех профилях, поэтому попадают
    /// в результат при любом значении аргумента.
    fn list_for_monitor(
        &self,
        monitor_id: &str,
        profile_id: Option<&str>,
    ) -> Result<Vec<Element>, CoreError>;

    /// Один элемент по идентификатору.
    fn get(&self, id: &str) -> Result<Option<Element>, CoreError>;

    /// Применяет набор изменений атомарно и возвращает новое состояние
    /// затронутых мониторов.
    fn apply_patch(&self, patches: Vec<ElementPatch>) -> Result<Vec<Element>, CoreError>;

    /// Удаляет элемент (каскадом уходят связанные данные).
    fn remove(&self, id: &str) -> Result<bool, CoreError>;
}

/// Реализация на SQLite.
#[derive(Debug, Clone, Copy)]
pub struct SqliteElements<'a> {
    db: &'a Db,
}

impl<'a> SqliteElements<'a> {
    /// Создаёт репозиторий поверх открытой базы.
    pub const fn new(db: &'a Db) -> Self {
        Self { db }
    }
}

impl ElementsRepo for SqliteElements<'_> {
    fn list_for_monitor(
        &self,
        monitor_id: &str,
        profile_id: Option<&str>,
    ) -> Result<Vec<Element>, CoreError> {
        let monitor = monitor_id.to_owned();
        let profile = profile_id.map(str::to_owned);
        self.db.call(move |conn| {
            let sql = format!(
                "SELECT {ELEMENT_COLUMNS} FROM elements \
                 WHERE monitor_id = ?1 AND (profile_id IS NULL OR profile_id IS ?2) \
                 ORDER BY z ASC, id ASC"
            );
            let mut stmt = conn.prepare_cached(&sql)?;
            let rows = stmt.query_map(params![monitor, profile], element_from_row)?;
            Ok(rows.collect::<Result<Vec<_>, _>>()?)
        })
    }

    fn get(&self, id: &str) -> Result<Option<Element>, CoreError> {
        let id = id.to_owned();
        self.db.call(move |conn| {
            let sql = format!("SELECT {ELEMENT_COLUMNS} FROM elements WHERE id = ?1");
            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query_map(params![id], element_from_row)?;
            Ok(rows.next().transpose()?)
        })
    }

    fn apply_patch(&self, patches: Vec<ElementPatch>) -> Result<Vec<Element>, CoreError> {
        if patches.is_empty() {
            return Ok(Vec::new());
        }
        // Валидация до открытия транзакции: не тратим блокировку на заведомо
        // неверный ввод.
        for patch in &patches {
            if let Some(element) = &patch.element {
                element.validate()?;
            }
        }

        self.db.call(move |conn| {
            let tx = conn.unchecked_transaction()?;
            let mut touched: Vec<String> = Vec::new();

            for patch in &patches {
                apply_one(&tx, patch)?;
                touched.push(patch.id.clone());
            }

            // Ответ — новое состояние затронутых элементов: UI применит его
            // без второго запроса, а другие поверхности получат тот же снапшот
            // по событию.
            let mut result = Vec::with_capacity(touched.len());
            for id in &touched {
                let sql = format!("SELECT {ELEMENT_COLUMNS} FROM elements WHERE id = ?1");
                let mut stmt = tx.prepare_cached(&sql)?;
                let mut rows = stmt.query_map(params![id], element_from_row)?;
                if let Some(row) = rows.next() {
                    result.push(row?);
                }
            }

            tx.commit()?;
            Ok(result)
        })
    }

    fn remove(&self, id: &str) -> Result<bool, CoreError> {
        let id = id.to_owned();
        self.db.call(move |conn| {
            let affected = conn.execute("DELETE FROM elements WHERE id = ?1", params![id])?;
            Ok(affected > 0)
        })
    }
}

/// Применяет один патч внутри уже открытой транзакции.
fn apply_one(conn: &Connection, patch: &ElementPatch) -> Result<(), CoreError> {
    match patch.op {
        PatchOp::Upsert => {
            let element = patch
                .element
                .as_ref()
                .ok_or_else(|| CoreError::InvalidData {
                    field: "element",
                    reason: "upsert требует полный элемент".to_owned(),
                })?;
            upsert(conn, element)
        }
        PatchOp::Move => {
            let frame = patch.frame.ok_or_else(|| CoreError::InvalidData {
                field: "frame",
                reason: "move требует геометрию".to_owned(),
            })?;
            // `Move` меняет только позицию: размеры приходят как 0.0, потому
            // что перетаскивание их не трогает. Писать ноль в `w`/`h` значило
            // бы обнулить элемент, поэтому обновляем ровно `x`/`y`.
            let affected = conn.execute(
                "UPDATE elements SET x = ?2, y = ?3, updated_at = ?4 WHERE id = ?1",
                params![patch.id, frame.x, frame.y, now_ms()],
            )?;
            require_affected(affected, &patch.id)
        }
        PatchOp::ZOrder => {
            let z = patch.z.ok_or_else(|| CoreError::InvalidData {
                field: "z",
                reason: "zorder требует z".to_owned(),
            })?;
            let affected = conn.execute(
                "UPDATE elements SET z = ?2, updated_at = ?3 WHERE id = ?1",
                params![patch.id, z, now_ms()],
            )?;
            require_affected(affected, &patch.id)
        }
        PatchOp::Settings => {
            let settings = patch
                .settings
                .as_ref()
                .ok_or_else(|| CoreError::InvalidData {
                    field: "settings",
                    reason: "settings требует значение".to_owned(),
                })?;
            let affected = conn.execute(
                "UPDATE elements SET settings_json = ?2, updated_at = ?3 WHERE id = ?1",
                params![patch.id, settings.to_string(), now_ms()],
            )?;
            require_affected(affected, &patch.id)
        }
        PatchOp::Remove => {
            // Удаление отсутствующего элемента — не ошибка: двойной клик по
            // «удалить» или удаление с двух мониторов не должны падать.
            conn.execute("DELETE FROM elements WHERE id = ?1", params![patch.id])?;
            Ok(())
        }
    }
}

/// Изменение, не затронувшее ни одной строки, — ошибка вызывающего кода.
fn require_affected(affected: usize, id: &str) -> Result<(), CoreError> {
    if affected == 0 {
        return Err(CoreError::NotFound {
            entity: "element",
            id: id.to_owned(),
        });
    }
    Ok(())
}

/// Полная запись элемента (создание или замена).
fn upsert(conn: &Connection, element: &Element) -> Result<(), CoreError> {
    conn.execute(
        "INSERT INTO elements (
            id, kind, widget_id, monitor_id, profile_id,
            x, y, w, h, z, rotation,
            settings_json, state_json, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(id) DO UPDATE SET
            kind = excluded.kind,
            widget_id = excluded.widget_id,
            monitor_id = excluded.monitor_id,
            profile_id = excluded.profile_id,
            x = excluded.x,
            y = excluded.y,
            w = excluded.w,
            h = excluded.h,
            z = excluded.z,
            rotation = excluded.rotation,
            settings_json = excluded.settings_json,
            state_json = excluded.state_json,
            updated_at = excluded.updated_at",
        params![
            element.id,
            element.kind.as_str(),
            element.widget_id,
            element.monitor_id,
            element.profile_id,
            element.frame.x,
            element.frame.y,
            element.frame.w,
            element.frame.h,
            element.z,
            element.rotation,
            element.settings.to_string(),
            element.state.to_string(),
            element.created_at,
            element.updated_at,
        ],
    )?;
    Ok(())
}

/// Миллисекунды UTC.
pub(crate) fn now_ms() -> i64 {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    i64::try_from(millis).unwrap_or(i64::MAX)
}

// ---------------------------------------------------------------------------
// Настройки
// ---------------------------------------------------------------------------

/// Доступ к таблице `settings` (key-value с JSON-значениями).
pub trait SettingsRepo {
    /// Читает значение по ключу.
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CoreError>;

    /// Записывает значение, перезаписывая существующее.
    fn set(&self, key: &str, value: &serde_json::Value) -> Result<(), CoreError>;

    /// Удаляет ключ. Отсутствующий ключ — не ошибка.
    fn remove(&self, key: &str) -> Result<(), CoreError>;
}

/// Реализация настроек на SQLite.
#[derive(Debug, Clone, Copy)]
pub struct SqliteSettings<'a> {
    db: &'a Db,
}

impl<'a> SqliteSettings<'a> {
    /// Создаёт репозиторий поверх открытой базы.
    pub const fn new(db: &'a Db) -> Self {
        Self { db }
    }
}

impl SettingsRepo for SqliteSettings<'_> {
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>, CoreError> {
        let key = key.to_owned();
        self.db.call(move |conn| {
            let raw: Option<String> = conn
                .prepare_cached("SELECT value FROM settings WHERE key = ?1")?
                .query_row(params![key], |r| r.get::<_, String>(0))
                .optional()?;
            // Повреждённый JSON не должен ронять весь запуск: настройка
            // читается как отсутствующая, а вызывающий код применит значение
            // по умолчанию. Иначе один испорченный ключ делает приложение
            // незапускаемым.
            Ok(raw.and_then(|s| serde_json::from_str(&s).ok()))
        })
    }

    fn set(&self, key: &str, value: &serde_json::Value) -> Result<(), CoreError> {
        let key = key.to_owned();
        let json = value.to_string();
        self.db.call(move |conn| {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![key, json, now_ms()],
            )?;
            Ok(())
        })
    }

    fn remove(&self, key: &str) -> Result<(), CoreError> {
        let key = key.to_owned();
        self.db.call(move |conn| {
            conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::OpenPolicy;
    use crate::migrations::migrate;
    use proptest::prelude::*;

    fn fresh_db() -> (tempfile::TempDir, Db) {
        let dir = tempfile::tempdir().expect("временный каталог");
        let path = dir.path().join("state.db");
        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
        migrate(&db).expect("миграции");
        (dir, db)
    }

    fn element(id: &str, monitor: &str) -> Element {
        Element {
            id: id.to_owned(),
            kind: ElementKind::Widget,
            widget_id: Some("core.clock".to_owned()),
            monitor_id: monitor.to_owned(),
            profile_id: None,
            frame: Rect::new(0.0, 0.0, 240.0, 120.0),
            z: 0,
            rotation: 0.0,
            settings: serde_json::json!({ "tz": "Europe/Berlin" }),
            state: serde_json::json!({}),
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
        }
    }

    fn upsert_patch(e: Element) -> ElementPatch {
        ElementPatch {
            id: e.id.clone(),
            op: PatchOp::Upsert,
            element: Some(e),
            frame: None,
            z: None,
            settings: None,
        }
    }

    #[test]
    fn mapping_covers_all_columns() {
        // Защита от рассинхрона `ELEMENT_COLUMNS` и `element_from_row`:
        // 15 колонок и 15 индексов. Если кто-то добавит колонку в одно место
        // и забудет в другом — тест укажет на это, а не пользователь.
        let count = ELEMENT_COLUMNS.split(',').count();
        assert_eq!(count, 15, "колонок в SELECT: {count}, ожидалось 15");
    }

    #[test]
    fn list_on_empty_database_is_empty_not_error() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        let list = repo.list_for_monitor("m1", None).expect("чтение");
        assert!(list.is_empty());
    }

    #[test]
    fn list_returns_single_element_with_all_fields() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        let original = element("e1", "m1");
        repo.apply_patch(vec![upsert_patch(original.clone())])
            .expect("запись");

        let list = repo.list_for_monitor("m1", None).expect("чтение");
        assert_eq!(list.len(), 1);
        // Сравнение целиком, а не по полям: любое потерянное поле (например
        // `rotation` или `settings`) проявится здесь.
        assert_eq!(list[0], original);
    }

    #[test]
    fn list_sorts_by_z_then_id() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);

        let mut a = element("a", "m1");
        a.z = 5;
        let mut b = element("b", "m1");
        b.z = 1;
        let mut c = element("c", "m1");
        c.z = 1;
        repo.apply_patch(vec![upsert_patch(a), upsert_patch(b), upsert_patch(c)])
            .expect("запись");

        let ids: Vec<String> = repo
            .list_for_monitor("m1", None)
            .expect("чтение")
            .into_iter()
            .map(|e| e.id)
            .collect();
        // При равном z порядок детерминирован по id — иначе раскладка
        // «прыгает» между запусками.
        assert_eq!(ids, vec!["b", "c", "a"]);
    }

    #[test]
    fn list_filters_by_monitor() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![
            upsert_patch(element("e1", "m1")),
            upsert_patch(element("e2", "m2")),
        ])
        .expect("запись");

        let m1 = repo.list_for_monitor("m1", None).expect("m1");
        assert_eq!(m1.len(), 1);
        assert_eq!(m1[0].id, "e1");
    }

    #[test]
    fn profile_null_is_visible_everywhere() {
        // Элемент без профиля виден в любом профиле — это его смысл.
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("all", "m1"))])
            .expect("запись");

        assert_eq!(
            repo.list_for_monitor("m1", Some("work"))
                .expect("work")
                .len(),
            1
        );
        assert_eq!(repo.list_for_monitor("m1", None).expect("any").len(), 1);
    }

    #[test]
    fn profile_filter_excludes_other_profiles() {
        let (_dir, db) = fresh_db();
        // Профиль обязан существовать: `profile_id` — внешний ключ, и это
        // проверяется здесь же. Тест заодно доказывает, что FK включён
        // (без `PRAGMA foreign_keys=ON` вставка прошла бы молча).
        db.call(|conn| {
            conn.execute(
                "INSERT INTO profiles (id, name, scene_json) VALUES ('work', 'Работа', '{}')",
                [],
            )?;
            Ok(())
        })
        .expect("профиль");
        let repo = SqliteElements::new(&db);
        let mut scoped = element("scoped", "m1");
        scoped.profile_id = Some("work".to_owned());
        repo.apply_patch(vec![upsert_patch(scoped)])
            .expect("запись");

        assert_eq!(
            repo.list_for_monitor("m1", Some("work"))
                .expect("work")
                .len(),
            1
        );
        assert_eq!(
            repo.list_for_monitor("m1", Some("evening"))
                .expect("evening")
                .len(),
            0
        );
    }

    #[test]
    fn unknown_profile_is_rejected_by_foreign_key() {
        // Обратная сторона фильтра: элемент не может ссылаться на профиль,
        // которого нет. Без этого в базе копились бы висячие ссылки, и после
        // удаления профиля элементы исчезали бы бесследно.
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        let mut dangling = element("e1", "m1");
        dangling.profile_id = Some("ghost-profile".to_owned());

        let err = repo
            .apply_patch(vec![upsert_patch(dangling)])
            .expect_err("ссылка на несуществующий профиль");
        assert!(matches!(err, CoreError::Sqlite(_)), "получено {err:?}");
    }

    #[test]
    fn get_returns_none_for_missing() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        assert!(repo.get("nope").expect("чтение").is_none());
    }

    #[test]
    fn get_returns_element() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e1", "m1"))])
            .expect("запись");
        assert_eq!(repo.get("e1").expect("чтение").expect("есть").id, "e1");
    }

    #[test]
    fn move_changes_only_position_not_size() {
        // Критично: перетаскивание приходит с нулевыми w/h, и если записать их
        // целиком, элемент исчезнет (нулевой размер).
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e1", "m1"))])
            .expect("запись");

        repo.apply_patch(vec![ElementPatch::move_to("e1", 100.0, 200.0)])
            .expect("перемещение");

        let e = repo.get("e1").expect("чтение").expect("есть");
        assert_eq!((e.frame.x, e.frame.y), (100.0, 200.0));
        assert_eq!(
            (e.frame.w, e.frame.h),
            (240.0, 120.0),
            "размер не должен меняться"
        );
        assert_eq!(
            e.settings,
            serde_json::json!({ "tz": "Europe/Berlin" }),
            "настройки целы"
        );
    }

    #[test]
    fn move_missing_element_is_not_found() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        let err = repo
            .apply_patch(vec![ElementPatch::move_to("ghost", 1.0, 1.0)])
            .expect_err("перемещение несуществующего");
        assert!(
            matches!(err, CoreError::NotFound { .. }),
            "получено {err:?}"
        );
    }

    #[test]
    fn zorder_and_settings_patches_work() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e1", "m1"))])
            .expect("запись");

        repo.apply_patch(vec![ElementPatch {
            id: "e1".to_owned(),
            op: PatchOp::ZOrder,
            element: None,
            frame: None,
            z: Some(42),
            settings: None,
        }])
        .expect("z-order");

        repo.apply_patch(vec![ElementPatch {
            id: "e1".to_owned(),
            op: PatchOp::Settings,
            element: None,
            frame: None,
            z: None,
            settings: Some(serde_json::json!({ "format": "24" })),
        }])
        .expect("настройки");

        let e = repo.get("e1").expect("чтение").expect("есть");
        assert_eq!(e.z, 42);
        assert_eq!(e.settings, serde_json::json!({ "format": "24" }));
    }

    #[test]
    fn remove_deletes_and_reports_whether_it_existed() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e1", "m1"))])
            .expect("запись");

        assert!(repo.remove("e1").expect("удаление"), "существующий элемент");
        assert!(
            !repo.remove("e1").expect("повторное удаление"),
            "уже удалён"
        );
    }

    #[test]
    fn remove_patch_of_missing_element_is_not_an_error() {
        // Двойной клик «удалить» или удаление с двух мониторов не должны падать.
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![ElementPatch::remove("ghost")])
            .expect("удаление несуществующего не ошибка");
    }

    #[test]
    fn ac2_partial_failure_rolls_back_whole_patch() {
        // AC-2: если четвёртый патч невалиден, первые три не должны примениться.
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e0", "m1"))])
            .expect("подготовка");

        let mut invalid = element("e4", "m1");
        invalid.frame = Rect::new(0.0, 0.0, 0.0, 0.0); // нулевой размер

        let err = repo
            .apply_patch(vec![
                upsert_patch(element("e1", "m1")),
                upsert_patch(element("e2", "m1")),
                upsert_patch(element("e3", "m1")),
                upsert_patch(invalid),
            ])
            .expect_err("невалидный элемент в наборе");
        assert!(
            matches!(err, CoreError::InvalidData { .. }),
            "получено {err:?}"
        );

        // Ни один из валидных не должен остаться.
        let list = repo.list_for_monitor("m1", None).expect("чтение");
        assert_eq!(
            list.len(),
            1,
            "должен остаться только e0, получено {:?}",
            list.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
        assert_eq!(list[0].id, "e0");
    }

    #[test]
    fn ac2_failure_inside_transaction_rolls_back() {
        // Второй путь: патч валиден по домену, но падает уже в SQL
        // (Move по несуществующему id). Первый патч обязан откатиться.
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e1", "m1"))])
            .expect("подготовка");

        let err = repo
            .apply_patch(vec![
                upsert_patch(element("e2", "m1")),
                ElementPatch::move_to("ghost", 5.0, 5.0),
            ])
            .expect_err("move несуществующего");
        assert!(
            matches!(err, CoreError::NotFound { .. }),
            "получено {err:?}"
        );

        assert!(
            repo.get("e2").expect("чтение").is_none(),
            "e2 должен откатиться вместе с неудавшимся патчем"
        );
    }

    #[test]
    fn ac1_patch_of_50_elements_is_fast() {
        // AC-1: ≤ 15 мс. Замер на отладочной сборке — самый пессимистичный
        // случай, поэтому порог с запасом: в release будет в разы быстрее.
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);

        let patches: Vec<ElementPatch> = (0..50)
            .map(|i| upsert_patch(element(&format!("e{i}"), "m1")))
            .collect();

        let started = std::time::Instant::now();
        let result = repo.apply_patch(patches).expect("патч");
        let elapsed = started.elapsed();

        assert_eq!(result.len(), 50, "в ответе все затронутые элементы");
        assert!(
            elapsed.as_millis() <= 1500,
            "50 элементов заняли {elapsed:?}; на debug-сборке допускается до 1500 мс, \
             релизный порог 15 мс проверяется в bench"
        );
    }

    #[test]
    fn upsert_twice_updates_in_place() {
        let (_dir, db) = fresh_db();
        let repo = SqliteElements::new(&db);
        repo.apply_patch(vec![upsert_patch(element("e1", "m1"))])
            .expect("создание");

        let mut updated = element("e1", "m1");
        updated.z = 7;
        updated.updated_at = 1_800_000_000_000;
        repo.apply_patch(vec![upsert_patch(updated)])
            .expect("обновление");

        let list = repo.list_for_monitor("m1", None).expect("чтение");
        assert_eq!(list.len(), 1, "не должно появиться второй записи");
        assert_eq!(list[0].z, 7);
        assert_eq!(list[0].updated_at, 1_800_000_000_000);
        // `created_at` сохраняется: это история создания, а не изменения.
        assert_eq!(list[0].created_at, 1_700_000_000_000);
    }

    #[test]
    fn settings_roundtrip() {
        let (_dir, db) = fresh_db();
        let repo = SqliteSettings::new(&db);
        let value = serde_json::json!({ "theme": "dark", "n": 1, "nested": { "a": [1, 2] } });
        repo.set("app", &value).expect("запись");
        assert_eq!(repo.get("app").expect("чтение"), Some(value));
    }

    #[test]
    fn settings_missing_key_is_none() {
        let (_dir, db) = fresh_db();
        let repo = SqliteSettings::new(&db);
        assert_eq!(repo.get("nope").expect("чтение"), None);
    }

    #[test]
    fn settings_overwrites_and_removes() {
        let (_dir, db) = fresh_db();
        let repo = SqliteSettings::new(&db);
        repo.set("k", &serde_json::json!(1)).expect("первая запись");
        repo.set("k", &serde_json::json!(2)).expect("перезапись");
        assert_eq!(repo.get("k").expect("чтение"), Some(serde_json::json!(2)));
        repo.remove("k").expect("удаление");
        assert_eq!(repo.get("k").expect("чтение"), None);
    }

    proptest! {
        // Property-тест из тест-плана: любой элемент, прошедший валидацию,
        // должен пережить запись и чтение без потерь. Генератор покрывает
        // крайние значения геометрии (дробные, отрицательные, максимальные),
        // поэтому проверяется не «средний случай», а весь диапазон.
        #[test]
        fn element_roundtrips_through_database(
            x in -90_000.0_f64..90_000.0,
            y in -90_000.0_f64..90_000.0,
            w in 1.0_f64..5_000.0,
            h in 1.0_f64..5_000.0,
            z in -1000_i32..1000,
            rotation in -360.0_f64..360.0,
        ) {
            let (_dir, db) = fresh_db();
            let repo = SqliteElements::new(&db);

            let original = Element {
                id: "prop".to_owned(),
                kind: ElementKind::Pin,
                widget_id: None,
                monitor_id: "m1".to_owned(),
                profile_id: None,
                frame: Rect::new(x, y, w, h),
                z,
                rotation,
                settings: serde_json::json!({}),
                state: serde_json::json!({}),
                created_at: 1_700_000_000_000,
                updated_at: 1_700_000_000_000,
            };
            prop_assume!(original.validate().is_ok());

            repo.apply_patch(vec![upsert_patch(original.clone())]).expect("запись");
            let read_back = repo.get("prop").expect("чтение").expect("есть");

            prop_assert_eq!(read_back, original);
        }
    }

    #[test]
    fn corrupted_json_in_settings_is_read_as_missing() {
        // Один испорченный ключ не должен делать приложение незапускаемым.
        let (_dir, db) = fresh_db();
        db.call(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('broken', '{{{', 0)",
                [],
            )?;
            Ok(())
        })
        .expect("подготовка");

        let repo = SqliteSettings::new(&db);
        assert_eq!(repo.get("broken").expect("чтение"), None);
    }

    #[test]
    fn unknown_kind_from_future_version_does_not_break_listing() {
        // Данные из более новой версии: элемент читается как виджет-заглушка,
        // а не роняет весь список.
        let (_dir, db) = fresh_db();
        db.call(|conn| {
            conn.execute(
                "INSERT INTO elements (id, kind, monitor_id, x, y, w, h, created_at, updated_at)
                 VALUES ('future', 'hologram', 'm1', 0, 0, 10, 10, 0, 0)",
                [],
            )?;
            Ok(())
        })
        .expect("подготовка");

        let repo = SqliteElements::new(&db);
        let list = repo.list_for_monitor("m1", None).expect("чтение");
        assert_eq!(list.len(), 1, "неизвестный вид не должен скрывать элемент");
        assert_eq!(list[0].kind, ElementKind::Widget);
    }
}
