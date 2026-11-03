//! Элемент холста и его геометрия.
//!
//! # Почему типы живут в `vela-core`, а не в UI
//!
//! Элемент — это данные, а не представление: его пишет Rust, читает UI через
//! IPC, и восстанавливать его после перезапуска должен тоже домен. Если
//! определить тип только в TypeScript, схема БД окажется без хозяина: любое
//! расхождение будет обнаруживаться в рантайме у пользователя.
//!
//! Полная валидация `settings` по JSON-схеме манифеста — задача ST-0302-01.
//! Здесь проверяется только то, что обязана проверять схема: геометрия и
//! согласованность полей между собой.

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Максимальная координата в мировых пикселях.
///
/// Не «на всякий случай», а следствие арифметики: при зуме 2.2 и координате
/// в миллион пикселей позиция на экране перестаёт помещаться в `f32`, которым
/// оперирует WebGL. Ограничение ставится на входе, чтобы не ловить `NaN`
/// в рендере.
pub const MAX_COORD: f64 = 100_000.0;

/// Максимальный размер элемента. Холст бесконечный, но элемент размером с
/// город — это ошибка ввода, а не намерение.
pub const MAX_SIZE: f64 = 20_000.0;

/// Прямоугольник в мировых координатах.
///
/// `f64`, а не `i32`: при зуме 0.35 элемент с шагом 8 px попадает на дробные
/// мировые координаты. Округление до целых на каждом шаге накапливало бы
/// дрейф при перетаскивании на больших расстояниях.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Левый край.
    pub x: f64,
    /// Верхний край.
    pub y: f64,
    /// Ширина.
    pub w: f64,
    /// Высота.
    pub h: f64,
}

impl Rect {
    /// Создаёт прямоугольник.
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Правый край.
    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    /// Нижний край.
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    /// Проверяет, что все координаты конечны и размеры положительны.
    pub fn is_valid(&self) -> bool {
        let finite =
            self.x.is_finite() && self.y.is_finite() && self.w.is_finite() && self.h.is_finite();
        if !finite {
            return false;
        }
        self.w > 0.0
            && self.h > 0.0
            && self.w <= MAX_SIZE
            && self.h <= MAX_SIZE
            && self.x.abs() <= MAX_COORD
            && self.y.abs() <= MAX_COORD
    }

    /// Пересекается ли с другим прямоугольником.
    ///
    /// Нужно для виртуализации (ST-0301-03): за вьюпортом ничего не рендерится.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }
}

/// Что именно лежит на холсте.
///
/// Набор намеренно не «закрыт» на уровне БД (`kind TEXT` без `CHECK`, см.
/// решение D-11): в P4 добавится `Group`, и каждая новая разновидность иначе
/// стоила бы миграции с перестройкой таблицы.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementKind {
    /// Виджет: часы, заметки, таймер и прочие модули.
    Widget,
    /// Закреплённый снимок экрана (P5).
    Pin,
    /// Стикер с текстом (P4).
    Note,
    /// Рисунок пером (P3).
    Ink,
    /// Группа элементов (P4).
    Group,
}

impl ElementKind {
    /// Строковое представление для колонки `kind`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Widget => "widget",
            Self::Pin => "pin",
            Self::Note => "note",
            Self::Ink => "ink",
            Self::Group => "group",
        }
    }

    /// Разбор значения из базы.
    ///
    /// Возвращает `None` для неизвестного вида: строка могла прийти из более
    /// новой версии приложения, и падать на этом нельзя.
    ///
    /// Вызывающий код пока заменяет `None` на `Widget` (D-14); после
    /// ST-0302-01 здесь появится `ElementKind::Unknown`.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "widget" => Some(Self::Widget),
            "pin" => Some(Self::Pin),
            "note" => Some(Self::Note),
            "ink" => Some(Self::Ink),
            "group" => Some(Self::Group),
            _ => None,
        }
    }
}

/// Элемент на холсте.
///
/// Одно определение для БД, IPC и UI: расхождение между слоями ловится
/// компилятором, а не тестами.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Идентификатор элемента.
    pub id: String,
    /// Что это.
    pub kind: ElementKind,
    /// Тип виджета (`core.clock`) — только для `kind == Widget`.
    pub widget_id: Option<String>,
    /// Монитор, на котором элемент лежит.
    pub monitor_id: String,
    /// Профиль, в котором элемент виден. `None` — во всех профилях.
    pub profile_id: Option<String>,
    /// Геометрия в мировых координатах.
    pub frame: Rect,
    /// Порядок наложения.
    pub z: i32,
    /// Поворот в градусах (для `pin` и `ink`).
    pub rotation: f64,
    /// Настройки модуля. Валидируются схемой манифеста (ST-0302-01).
    pub settings: serde_json::Value,
    /// Свободная зона состояния модуля. Лимит 64 КБ проверяется в домене.
    pub state: serde_json::Value,
    /// Время создания, миллисекунды UTC.
    pub created_at: i64,
    /// Время последнего изменения, миллисекунды UTC.
    pub updated_at: i64,
}

/// Максимальный размер `state` в байтах.
///
/// 64 КБ — из `09-widget-spec.md`. Без лимита модуль, складывающий туда
/// историю, за год вырастет в файл, который нельзя ни прочитать, ни отправить
/// в поддержку.
pub const MAX_STATE_BYTES: usize = 64 * 1024;

impl Element {
    /// Проверяет инварианты элемента.
    ///
    /// Возвращает ошибку с путём до поля — сообщение показывается в
    /// диагностике, поэтому «что именно не так» важнее кода.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.trim().is_empty() {
            return Err(CoreError::InvalidData {
                field: "id",
                reason: "пустой идентификатор".to_owned(),
            });
        }
        if self.monitor_id.trim().is_empty() {
            return Err(CoreError::InvalidData {
                field: "monitor_id",
                reason: "пустой идентификатор монитора".to_owned(),
            });
        }
        if !self.frame.is_valid() {
            return Err(CoreError::InvalidData {
                field: "frame",
                reason: format!(
                    "недопустимая геометрия: x={} y={} w={} h={}",
                    self.frame.x, self.frame.y, self.frame.w, self.frame.h
                ),
            });
        }
        // Согласованность вида и типа виджета: `Widget` без `widget_id` — это
        // элемент, который некому отрисовать, и он молча превратится в пустую
        // плашку. Лучше отказать на входе.
        match self.kind {
            ElementKind::Widget if self.widget_id.is_none() => {
                return Err(CoreError::InvalidData {
                    field: "widget_id",
                    reason: "для kind=widget обязателен widget_id".to_owned(),
                });
            }
            ElementKind::Widget => {}
            _ if self.widget_id.is_some() => {
                return Err(CoreError::InvalidData {
                    field: "widget_id",
                    reason: "widget_id допустим только при kind=widget".to_owned(),
                });
            }
            _ => {}
        }
        if !self.rotation.is_finite() || self.rotation.abs() > 3600.0 {
            return Err(CoreError::InvalidData {
                field: "rotation",
                reason: format!("недопустимый поворот: {}", self.rotation),
            });
        }
        let state_len = self.state.to_string().len();
        if state_len > MAX_STATE_BYTES {
            return Err(CoreError::InvalidData {
                field: "state",
                reason: format!("состояние {state_len} байт, лимит {MAX_STATE_BYTES}"),
            });
        }
        Ok(())
    }
}

/// Что сделать с элементом.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchOp {
    /// Создать или полностью заменить.
    Upsert,
    /// Изменить геометрию (частично: переданные поля).
    Move,
    /// Изменить порядок наложения.
    ZOrder,
    /// Заменить настройки.
    Settings,
    /// Удалить.
    Remove,
}

/// Один шаг изменения раскладки.
///
/// Патч, а не целый элемент: при перетаскивании меняются только `x`/`y`, и
/// передавать через IPC весь элемент с настройками и состоянием означало бы
/// гонять десятки килобайт на каждое движение мыши.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementPatch {
    /// Кого меняем.
    pub id: String,
    /// Что делаем.
    pub op: PatchOp,
    /// Новый элемент — для `Upsert`.
    pub element: Option<Element>,
    /// Новая геометрия — для `Upsert` и `Move`. Для `Move` достаточно `x`/`y`.
    pub frame: Option<Rect>,
    /// Новый `z` — для `ZOrder`.
    pub z: Option<i32>,
    /// Новые настройки — для `Settings`.
    pub settings: Option<serde_json::Value>,
}

impl ElementPatch {
    /// Изменение позиции: самый частый случай.
    pub fn move_to(id: impl Into<String>, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            op: PatchOp::Move,
            element: None,
            frame: Some(Rect::new(x, y, 0.0, 0.0)),
            z: None,
            settings: None,
        }
    }

    /// Удаление.
    pub fn remove(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            op: PatchOp::Remove,
            element: None,
            frame: None,
            z: None,
            settings: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_element() -> Element {
        Element {
            id: "e1".to_owned(),
            kind: ElementKind::Widget,
            widget_id: Some("core.clock".to_owned()),
            monitor_id: "m1".to_owned(),
            profile_id: None,
            frame: Rect::new(10.0, 20.0, 240.0, 120.0),
            z: 0,
            rotation: 0.0,
            settings: serde_json::json!({}),
            state: serde_json::json!({}),
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_000_000,
        }
    }

    #[test]
    fn valid_element_passes_validation() {
        assert!(valid_element().validate().is_ok());
    }

    #[test]
    fn empty_id_is_rejected() {
        let mut e = valid_element();
        e.id = "   ".to_owned();
        let err = e.validate().expect_err("пустой id недопустим");
        assert!(matches!(err, CoreError::InvalidData { field: "id", .. }));
    }

    #[test]
    fn empty_monitor_is_rejected() {
        let mut e = valid_element();
        e.monitor_id = String::new();
        let err = e.validate().expect_err("пустой монитор недопустим");
        assert!(matches!(
            err,
            CoreError::InvalidData {
                field: "monitor_id",
                ..
            }
        ));
    }

    #[test]
    fn widget_must_have_widget_id() {
        let mut e = valid_element();
        e.widget_id = None;
        let err = e.validate().expect_err("виджет без типа недопустим");
        assert!(matches!(
            err,
            CoreError::InvalidData {
                field: "widget_id",
                ..
            }
        ));
    }

    #[test]
    fn non_widget_must_not_have_widget_id() {
        // Снимок с widget_id — это противоречие: одно из полей точно лишнее,
        // и молча его игнорировать значит прятать ошибку в вызывающем коде.
        let mut e = valid_element();
        e.kind = ElementKind::Pin;
        let err = e.validate().expect_err("pin не может быть виджетом");
        assert!(matches!(
            err,
            CoreError::InvalidData {
                field: "widget_id",
                ..
            }
        ));
    }

    #[test]
    fn non_finite_geometry_is_rejected() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut e = valid_element();
            e.frame = Rect::new(bad, 0.0, 100.0, 100.0);
            assert!(e.validate().is_err(), "координата {bad} должна отвергаться");
        }
    }

    #[test]
    fn zero_and_negative_sizes_are_rejected() {
        let mut e = valid_element();
        e.frame = Rect::new(0.0, 0.0, 0.0, 100.0);
        assert!(e.validate().is_err(), "нулевая ширина");

        let mut e = valid_element();
        e.frame = Rect::new(0.0, 0.0, -50.0, 100.0);
        assert!(e.validate().is_err(), "отрицательная ширина");
    }

    #[test]
    fn geometry_beyond_limits_is_rejected() {
        let mut e = valid_element();
        e.frame = Rect::new(MAX_COORD + 1.0, 0.0, 100.0, 100.0);
        assert!(e.validate().is_err(), "координата за пределом");

        let mut e = valid_element();
        e.frame = Rect::new(0.0, 0.0, MAX_SIZE + 1.0, 100.0);
        assert!(e.validate().is_err(), "размер за пределом");
    }

    #[test]
    fn oversized_state_is_rejected() {
        // Лимит из 09-widget-spec.md: модуль не должен превращать БД в свалку.
        let mut e = valid_element();
        e.state = serde_json::Value::String("x".repeat(MAX_STATE_BYTES));
        let err = e.validate().expect_err("состояние сверх лимита");
        assert!(matches!(err, CoreError::InvalidData { field: "state", .. }));
    }

    #[test]
    fn rotation_out_of_range_is_rejected() {
        let mut e = valid_element();
        e.rotation = 7200.0;
        assert!(e.validate().is_err());
    }

    #[test]
    fn kind_roundtrips_through_string() {
        for kind in [
            ElementKind::Widget,
            ElementKind::Pin,
            ElementKind::Note,
            ElementKind::Ink,
            ElementKind::Group,
        ] {
            assert_eq!(ElementKind::from_str_opt(kind.as_str()), Some(kind));
        }
    }

    #[test]
    fn unknown_kind_is_none_not_panic() {
        // Значение из более новой версии не должно ронять чтение раскладки.
        assert_eq!(ElementKind::from_str_opt("hologram"), None);
        assert_eq!(ElementKind::from_str_opt(""), None);
    }

    #[test]
    fn rect_intersection_works_for_virtualization() {
        let viewport = Rect::new(0.0, 0.0, 1000.0, 800.0);
        assert!(viewport.intersects(&Rect::new(900.0, 700.0, 200.0, 200.0)));
        assert!(!viewport.intersects(&Rect::new(1001.0, 0.0, 100.0, 100.0)));
        // Касание краем — не пересечение: элемент за границей не рисуется.
        assert!(!viewport.intersects(&Rect::new(1000.0, 0.0, 100.0, 100.0)));
        // Отрицательные координаты — норма: холст бесконечный в обе стороны.
        assert!(viewport.intersects(&Rect::new(-50.0, -50.0, 100.0, 100.0)));
    }

    #[test]
    fn rect_edges_are_computed_not_stored() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(r.right(), 110.0);
        assert_eq!(r.bottom(), 70.0);
    }

    #[test]
    fn json_shape_is_camel_free_and_stable() {
        // Контракт с UI: имена полей фиксированы, `frame` — вложенный объект.
        // Тест ловит случайное переименование при рефакторинге.
        let json = serde_json::to_value(valid_element()).expect("сериализация");
        let obj = json.as_object().expect("объект");
        for key in [
            "id",
            "kind",
            "widget_id",
            "monitor_id",
            "profile_id",
            "frame",
            "z",
            "rotation",
            "settings",
            "state",
            "created_at",
            "updated_at",
        ] {
            assert!(obj.contains_key(key), "нет поля {key} в {json}");
        }
        assert_eq!(json["kind"], serde_json::json!("widget"));
        assert_eq!(json["frame"]["w"], serde_json::json!(240.0));
    }
}
