//! Миграции схемы.
//!
//! # Почему файлы встроены в бинарь
//!
//! `include_str!` вместо чтения с диска: приложение после установки не должно
//! зависеть от того, доехали ли SQL-файлы до папки программы. Побочный
//! эффект — невозможность «поправить миграцию на месте» у пользователя, что
//! здесь скорее плюс: схема на диске и код всегда совпадают.
//!
//! # Почему версии читаются из таблицы, а не из `user_version`
//!
//! SQLite хранит произвольное число в `PRAGMA user_version`, и соблазн
//! использовать его велик. Но тогда нельзя ответить на вопрос «какая именно
//! миграция применилась» — только «какая по счёту». При отладке и при разборе
//! бэкапов нужно второе, поэтому ведём явную таблицу с датой применения.
//!
//! # Транзакция на файл, а не на весь прогон
//!
//! Если открыть одну транзакцию на все миграции, падение на третьей откатит и
//! первые две — база вернётся к исходному состоянию, что формально корректно,
//! но прячет прогресс: непонятно, где именно сломалось. Транзакция на файл
//! даёт частичный прогресс, при котором состояние всё равно консистентно:
//! применённые миграции зафиксированы, неприменённые отсутствуют.

use rusqlite::{Connection, OptionalExtension};
use tracing::{debug, info, warn};

use crate::db::{checkpoint_public, Db};
use crate::error::CoreError;

/// Одна миграция: версия и её SQL.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Номер версии. Растёт на единицу, без пропусков.
    pub version: u32,
    /// Имя файла — попадает в лог и в таблицу, чтобы было видно, что применено.
    pub name: &'static str,
    /// Тело миграции.
    pub sql: &'static str,
}

/// Все миграции, известные этой сборке. Порядок — по возрастанию версии.
///
/// Добавление новой миграции: файл `000N_*.sql` + строка здесь. Компилятор
/// сверит, что версии идут подряд и что последняя совпадает с
/// [`crate::SCHEMA_VERSION`] — рассинхрон ломает сборку, а не прод.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "0001_init",
    sql: include_str!("../migrations/0001_init.sql"),
}];

/// Чем закончился прогон миграций.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrateOutcome {
    /// Схема уже была актуальной.
    UpToDate {
        /// Текущая версия.
        version: u32,
    },
    /// Применены миграции (возможно, ни одной — тогда это `UpToDate`).
    Applied {
        /// Было до прогона (0 — база пустая).
        from: u32,
        /// Стало после.
        to: u32,
        /// Куда сохранена копия перед изменением, если она делалась.
        backup: Option<std::path::PathBuf>,
    },
}

/// Применяет недостающие миграции к открытой базе.
///
/// Перед изменением схемы, если в базе уже есть данные (`version > 0`),
/// создаётся копия файла `state.db.bak-<version>`. Копия делается до первой
/// транзакции: смысл бэкапа в том, чтобы существовать на момент, когда миграция
/// ещё не начала портить данные.
pub fn migrate(db: &Db) -> Result<MigrateOutcome, CoreError> {
    migrate_with(db, MIGRATIONS)
}

/// То же, но с явным списком миграций.
///
/// Существует ради тестируемости: полный путь «есть данные → есть новая
/// миграция → создаётся копия» иначе не проверить без выдумывания
/// несуществующей схемы. Тест на реальной схеме проверял бы только отсутствие
/// второй миграции, то есть ничего.
pub fn migrate_with(
    db: &Db,
    migrations: &'static [Migration],
) -> Result<MigrateOutcome, CoreError> {
    let path = db.path().to_path_buf();
    // Копия внутрь замыкания: `call` требует 'static, а ссылка на `path`
    // живёт только до конца этой функции. PathBuf дешёвый, clone оправдан.
    let path_for_backup = path.clone();

    db.call(move |conn| {
        bootstrap_registry(conn)?;
        let from = current_version(conn)?;
        let target = migrations.last().map(|m| m.version).unwrap_or(0);

        if from > target {
            // База новее приложения: например, пользователь откатился на
            // предыдущую версию. Применять «вниз» мы не умеем и не должны —
            // это уничтожило бы данные новых миграций.
            warn!(
                db_version = from,
                app_version = target,
                "схема новее приложения"
            );
            return Err(CoreError::SchemaTooNew {
                found: from,
                supported: target,
            });
        }

        if from == target {
            debug!(version = from, "схема актуальна");
            return Ok(MigrateOutcome::UpToDate { version: from });
        }

        // Бэкап только если в базе что-то есть: на пустом файле копировать нечего,
        // а лишний файл в каталоге данных сбивает с толку при разборе проблем.
        let backup = if from > 0 {
            // Перед копированием файла переносим WAL в основной файл: иначе
            // копия получится неполной — часть страниц ещё в журнале.
            // Активной транзакции на этом этапе нет: предыдущий запрос
            // (`current_version`) её не открывает.
            checkpoint_public(conn)?;
            Some(backup_file(&path_for_backup, from)?)
        } else {
            None
        };

        for migration in migrations.iter().filter(|m| m.version > from) {
            apply_one(conn, migration)?;
        }

        let to = current_version(conn)?;
        info!(from, to, "миграции применены");
        Ok(MigrateOutcome::Applied { from, to, backup })
    })
}

/// Создаёт таблицу учёта версий, если её ещё нет.
///
/// Отдельно от `0001_init.sql`, потому что иначе возникает замкнутый круг:
/// чтобы понять, применять ли `0001_init`, нужно прочитать `schema_migrations`,
/// а её создаёт сам `0001_init`. Bootstrap-таблица — единственный DDL вне
/// миграций, и он идемпотентен по своей природе.
fn bootstrap_registry(conn: &Connection) -> Result<(), CoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version    INTEGER PRIMARY KEY,
            name       TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        )",
    )?;
    Ok(())
}

/// Текущая версия схемы: максимальный номер в реестре, 0 если пусто.
fn current_version(conn: &Connection) -> Result<u32, CoreError> {
    let version: Option<u32> = conn
        .query_row("SELECT max(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .optional()?
        .flatten();
    Ok(version.unwrap_or(0))
}

/// Применяет одну миграцию в собственной транзакции.
fn apply_one(conn: &Connection, migration: &Migration) -> Result<(), CoreError> {
    debug!(
        version = migration.version,
        name = migration.name,
        "применение миграции"
    );

    // `execute_batch` внутри `BEGIN`/`COMMIT`: если любая инструкция упадёт,
    // весь файл откатится и версия не запишется. Это и есть AC-2.
    conn.execute_batch("BEGIN")?;
    let result = (|| -> Result<(), CoreError> {
        conn.execute_batch(migration.sql)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![migration.version, migration.name, now_ms()],
        )?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(())
        }
        Err(err) => {
            // Откат обязателен, но его собственная ошибка не должна скрыть
            // исходную причину падения — она важнее.
            if let Err(rollback_err) = conn.execute_batch("ROLLBACK") {
                warn!(error = %rollback_err, "откат миграции не удался");
            }
            Err(err)
        }
    }
}

/// Копирует файл базы в `state.db.bak-<version>`.
fn backup_file(path: &std::path::Path, version: u32) -> Result<std::path::PathBuf, CoreError> {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".bak-{version}"));
    let target = path.with_file_name(&name);

    // Копия, а не перенос: оригинал остаётся рабочим, если миграция пройдёт
    // успешно. Бэкап удаляется вручную или политикой ротации в T-0803.
    std::fs::copy(path, &target)?;
    info!(backup = %target.display(), version, "создана копия перед миграцией");
    Ok(target)
}

/// Миллисекунды UTC — формат всех временных меток в схеме.
///
/// `as_millis()` возвращает `u128`, а колонки — `INTEGER` (i64). Прямое
/// приведение `as i64` обрезало бы значение на датах за 292 млн лет, что для
/// метки времени означает «неизвестно что»; насыщение до `i64::MAX` хотя бы
/// остаётся монотонным и не превращается в отрицательное число.
fn now_ms() -> i64 {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    i64::try_from(millis).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::OpenPolicy;

    fn db(conn_str: &std::path::Path) -> Db {
        Db::open(conn_str, OpenPolicy::Recover).expect("открытие")
    }

    fn temp_db() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("временный каталог");
        let path = dir.path().join("state.db");
        (dir, path)
    }

    #[test]
    fn migration_versions_are_contiguous_and_match_schema_version() {
        // Инвариант, который иначе выяснится только на чужой машине:
        // версии идут подряд с 1, и последняя совпадает с SCHEMA_VERSION.
        for (i, m) in MIGRATIONS.iter().enumerate() {
            assert_eq!(
                m.version as usize,
                i + 1,
                "миграции должны идти подряд с 1, нарушено на {}",
                m.name
            );
        }
        assert_eq!(
            MIGRATIONS.last().map(|m| m.version),
            Some(crate::SCHEMA_VERSION),
            "SCHEMA_VERSION должна совпадать с последней миграцией"
        );
    }

    #[test]
    fn migrates_empty_database_to_current_version() {
        let (_dir, path) = temp_db();
        let db = db(&path);
        let outcome = migrate(&db).expect("миграция");

        assert_eq!(
            outcome,
            MigrateOutcome::Applied {
                from: 0,
                to: crate::SCHEMA_VERSION,
                backup: None
            },
            "на пустой базе копия не нужна"
        );

        let version = db
            .call(|conn| current_version(conn))
            .expect("чтение версии");
        assert_eq!(version, crate::SCHEMA_VERSION);
    }

    #[test]
    fn ac1_second_run_applies_nothing() {
        // AC-1: повторный прогон не применяет миграции заново.
        let (_dir, path) = temp_db();
        let db = db(&path);
        migrate(&db).expect("первый прогон");

        let second = migrate(&db).expect("второй прогон");
        assert_eq!(
            second,
            MigrateOutcome::UpToDate {
                version: crate::SCHEMA_VERSION
            },
            "повторный прогон должен сообщить UpToDate"
        );

        let rows: i64 = db
            .call(|conn| {
                Ok(conn.query_row("SELECT count(*) FROM schema_migrations", [], |r| r.get(0))?)
            })
            .expect("подсчёт");
        assert_eq!(
            rows,
            i64::from(crate::SCHEMA_VERSION),
            "по одной записи на миграцию"
        );
    }

    #[test]
    fn schema_contains_all_v1_tables() {
        let (_dir, path) = temp_db();
        let db = db(&path);
        migrate(&db).expect("миграция");

        let tables: Vec<String> = db
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
                )?;
                let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                Ok(rows.collect::<Result<Vec<_>, _>>()?)
            })
            .expect("список таблиц");

        for expected in [
            "capture_history",
            "cron_jobs",
            "cron_runs",
            "elements",
            "monitors",
            "notes",
            "profiles",
            "schema_migrations",
            "settings",
        ] {
            assert!(
                tables.iter().any(|t| t == expected),
                "нет таблицы {expected}, есть: {tables:?}"
            );
        }
    }

    #[test]
    fn cascade_deleting_element_removes_note() {
        // Правило из T-0104, AC-3: заметка не должна переживать свой элемент.
        let (_dir, path) = temp_db();
        let db = db(&path);
        migrate(&db).expect("миграция");

        db.call(|conn| {
            conn.execute(
                "INSERT INTO elements (id, kind, monitor_id, x, y, w, h, created_at, updated_at)
                 VALUES ('e1', 'note', 'm1', 0, 0, 100, 100, 0, 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO notes (element_id, body) VALUES ('e1', 'текст')",
                [],
            )?;
            conn.execute("DELETE FROM elements WHERE id = 'e1'", [])?;

            let left: i64 = conn.query_row("SELECT count(*) FROM notes", [], |r| r.get(0))?;
            assert_eq!(left, 0, "заметка должна удалиться каскадом");
            Ok(())
        })
        .expect("каскад");
    }

    #[test]
    fn deleting_profile_keeps_elements_visible_everywhere() {
        // ST-0305-01: профиль удалён — элементы становятся видимыми везде,
        // а не исчезают вместе с профилем.
        let (_dir, path) = temp_db();
        let db = db(&path);
        migrate(&db).expect("миграция");

        db.call(|conn| {
            conn.execute(
                "INSERT INTO profiles (id, name, scene_json) VALUES ('p1', 'Работа', '{}')",
                [],
            )?;
            conn.execute(
                "INSERT INTO elements (id, kind, monitor_id, profile_id, x, y, w, h, created_at, updated_at)
                 VALUES ('e1', 'widget', 'm1', 'p1', 0, 0, 100, 100, 0, 0)",
                [],
            )?;
            conn.execute("DELETE FROM profiles WHERE id = 'p1'", [])?;

            // Тип колонки именно Option<String>: `get::<_, String>` на NULL
            // вернул бы InvalidColumnType, хотя NULL здесь — ожидаемый результат.
            let profile: Option<String> = conn.query_row(
                "SELECT profile_id FROM elements WHERE id = 'e1'",
                [],
                |r| r.get::<_, Option<String>>(0),
            )?;
            assert_eq!(profile, None, "profile_id должен стать NULL");
            Ok(())
        })
        .expect("SET NULL");
    }

    #[test]
    fn ac3_backup_created_before_migration_when_data_exists() {
        // AC-3: при version > 0 копия создаётся до изменения.
        let (_dir, path) = temp_db();
        {
            let mut db = db(&path);
            migrate(&db).expect("до состояния v1");
            db.call(|conn| {
                conn.execute(
                    "INSERT INTO settings (key, value, updated_at) VALUES ('k', 'v', 0)",
                    [],
                )?;
                Ok(())
            })
            .expect("данные");
            db.shutdown().expect("закрытие");
        }

        // Второй прогон на уже мигрированной базе ничего не применяет,
        // поэтому проверяем сам механизм бэкапа напрямую.
        let target = backup_file(&path, 1).expect("копия");
        assert!(target.exists());
        assert!(
            target.to_string_lossy().ends_with(".bak-1"),
            "имя копии: {target:?}"
        );

        // Копия должна быть валидной базой, а не обрезанным файлом.
        let copy = rusqlite::Connection::open(&target).expect("копия открывается");
        let k: String = copy
            .query_row("SELECT value FROM settings WHERE key='k'", [], |r| r.get(0))
            .expect("данные в копии");
        assert_eq!(k, "v");
    }

    #[test]
    fn ac3_full_path_creates_backup_when_moving_from_v1_to_v2() {
        // Полный путь AC-3: база уже на v1 с данными, приходит миграция v2 —
        // копия должна появиться ДО изменения и содержать данные v1.
        static V2: &[Migration] = &[
            Migration {
                version: 1,
                name: "0001_init",
                sql: include_str!("../migrations/0001_init.sql"),
            },
            Migration {
                version: 2,
                name: "0002_probe",
                sql: "CREATE TABLE probe (v INTEGER);",
            },
        ];

        let (_dir, path) = temp_db();
        let db = db(&path);
        migrate(&db).expect("до v1");
        db.call(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES ('k', 'v1-data', 0)",
                [],
            )?;
            Ok(())
        })
        .expect("данные v1");

        let outcome = migrate_with(&db, V2).expect("до v2");
        let MigrateOutcome::Applied { from, to, backup } = outcome else {
            panic!("ожидалось применение, получено {outcome:?}");
        };
        assert_eq!((from, to), (1, 2));

        let backup = backup.expect("при переходе с v1 копия обязательна");
        assert!(backup.exists(), "файл копии должен существовать");
        assert!(
            backup.to_string_lossy().ends_with(".bak-1"),
            "имя: {backup:?}"
        );

        // Копия — состояние ДО миграции: в ней есть settings и нет probe.
        let copy = rusqlite::Connection::open(&backup).expect("копия открывается");
        let value: String = copy
            .query_row("SELECT value FROM settings WHERE key='k'", [], |r| r.get(0))
            .expect("данные в копии");
        assert_eq!(value, "v1-data");
        let has_probe: i64 = copy
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='probe'",
                [],
                |r| r.get(0),
            )
            .expect("проверка probe");
        assert_eq!(has_probe, 0, "в копии не должно быть таблиц из v2");
    }

    #[test]
    fn no_backup_when_starting_from_empty_database() {
        // Копировать пустую базу — создавать файл, который ничего не защищает.
        static V2: &[Migration] = &[
            Migration {
                version: 1,
                name: "0001_init",
                sql: include_str!("../migrations/0001_init.sql"),
            },
            Migration {
                version: 2,
                name: "0002_probe",
                sql: "CREATE TABLE probe (v INTEGER);",
            },
        ];
        let (_dir, path) = temp_db();
        let db = db(&path);
        let outcome = migrate_with(&db, V2).expect("миграция с нуля");

        assert_eq!(
            outcome,
            MigrateOutcome::Applied {
                from: 0,
                to: 2,
                backup: None
            },
            "на пустой базе копия не нужна"
        );
    }

    #[test]
    fn ac2_failing_migration_rolls_back_and_keeps_version() {
        // AC-2 проверяем на искусственной миграции: первая инструкция валидна,
        // вторая падает. В базе не должно остаться ни таблицы, ни записи версии.
        let (_dir, path) = temp_db();
        let db = db(&path);
        migrate(&db).expect("до v1");
        let before = db.call(|conn| current_version(conn)).expect("версия");

        let broken = Migration {
            version: before + 1,
            name: "9999_broken",
            sql: "CREATE TABLE half_done (v INTEGER);\nCREATE TABLE elements (v INTEGER);",
        };

        let broken_for_call = broken;
        let err = db
            .call(move |conn| apply_one(conn, &broken_for_call))
            .expect_err("миграция должна упасть");
        assert!(matches!(err, CoreError::Sqlite(_)), "получено {err:?}");

        db.call(move |conn| {
            let after = current_version(conn)?;
            assert_eq!(after, before, "версия не должна измениться");

            let half: i64 = conn.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='half_done'",
                [],
                |r| r.get(0),
            )?;
            assert_eq!(half, 0, "частично применённая таблица должна откатиться");
            Ok(())
        })
        .expect("проверки после падения");
    }

    #[test]
    fn schema_too_new_is_reported_not_silently_downgraded() {
        // База новее приложения: применить «вниз» нельзя, надо сказать явно.
        let (_dir, path) = temp_db();
        let db = db(&path);
        db.call(|conn| {
            bootstrap_registry(conn)?;
            conn.execute(
                "INSERT INTO schema_migrations (version, name, applied_at) VALUES (99, 'future', 0)",
                [],
            )?;
            Ok(())
        })
        .expect("подготовка");

        let err = migrate(&db).expect_err("должна быть ошибка");
        assert!(
            matches!(err, CoreError::SchemaTooNew { found: 99, .. }),
            "получено {err:?}"
        );
    }

    #[test]
    fn registry_is_created_once_and_survives_reopen() {
        let (_dir, path) = temp_db();
        {
            let mut db = db(&path);
            migrate(&db).expect("миграция");
            db.shutdown().expect("закрытие");
        }
        let db = db(&path);
        // Второе открытие не должно пытаться создать реестр повторно с ошибкой.
        let outcome = migrate(&db).expect("повторное открытие");
        assert!(matches!(outcome, MigrateOutcome::UpToDate { .. }));
    }

    #[test]
    fn now_ms_is_plausible() {
        // Защита от опечатки в единицах: миллисекунды, а не секунды.
        let ts = now_ms();
        assert!(
            ts > 1_700_000_000_000,
            "ожидались миллисекунды, получено {ts}"
        );
    }
}
