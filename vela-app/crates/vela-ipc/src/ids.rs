//! Типизированные идентификаторы.
//!
//! Голые `String` запрещены в публичных контрактах: домен знает пять видов
//! идентификаторов, и каждый из них живёт в своей таблице. Newtype-обёртки
//! дают две вещи, которых не даёт `String`:
//!
//! 1. Компилятор не даст передать `MonitorId` туда, где ждут `ElementId`.
//! 2. Все они сериализуются как строка, поэтому на стороне TypeScript это
//!    обычные `string` с разными именами типов — читаемо и без накладных расходов.
//!
//! Идентификаторы **неизменяемы**: они попадают в `elements.id`, в файлы
//! ассетов и в журналы. Формат — только ASCII, нижний регистр, `[a-z0-9_-]`,
//! длина 1..=64. Валидация выполняется при десериализации, а не при записи в БД:
//! тогда невалидное значение не может проникнуть в домен вообще.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use ts_rs::TS;

/// Ошибка разбора идентификатора.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    /// Пустая строка.
    #[error("identifier must not be empty")]
    Empty,
    /// Длина больше допустимой.
    #[error("identifier is {len} bytes long, maximum is {max}")]
    TooLong {
        /// Фактическая длина.
        len: usize,
        /// Максимум.
        max: usize,
    },
    /// Недопустимый символ: только `a-z`, `0-9`, `_`, `-`.
    #[error("identifier contains invalid character {ch:?} at byte {index}")]
    InvalidChar {
        /// Найденный символ.
        ch: char,
        /// Позиция в строке.
        index: usize,
    },
}

const MAX_ID_LEN: usize = 64;

/// Проверяет строку и возвращает `true`, если она — допустимый идентификатор.
///
/// Вынесено отдельно, чтобы UI-валидация и Rust-валидация не разъезжались.
fn is_valid_id(value: &str) -> Result<(), IdError> {
    if value.is_empty() {
        return Err(IdError::Empty);
    }
    if value.len() > MAX_ID_LEN {
        return Err(IdError::TooLong {
            len: value.len(),
            max: MAX_ID_LEN,
        });
    }
    for (index, ch) in value.char_indices() {
        let ok = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-';
        if !ok {
            return Err(IdError::InvalidChar { ch, index });
        }
    }
    Ok(())
}

/// Макрос, чтобы пять одинаковых типов не размножали код вручную:
/// любая рассинхронизация валидации между ними была бы незаметной ошибкой.
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, TS)]
        #[ts(type = "string")]
        pub struct $name(String);

        impl $name {
            /// Создаёт идентификатор, проверяя формат.
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                is_valid_id(&value)?;
                Ok(Self(value))
            }

            /// Возвращает строковое представление.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = IdError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;
            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        // Сериализуем как прозрачную строку: на стороне TS это `string`.
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        // Десериализация валидирует: невалидный id не может появиться в домене
        // даже из повреждённого JSON в БД.
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(deserializer)?;
                Self::new(raw).map_err(serde::de::Error::custom)
            }
        }
    };
}

id_type!(
    /// Идентификатор элемента холста (`elements.id`).
    ElementId
);
id_type!(
    /// Идентификатор монитора (`monitors.id`). Стабилен между запусками,
    /// пока монитор не переподключён с другими параметрами.
    MonitorId
);
id_type!(
    /// Идентификатор профиля (`profiles.id`).
    ProfileId
);
id_type!(
    /// Идентификатор типа виджета, формат `namespace.kebab`, например
    /// `core.clock`. Точка разрешена, поэтому валидация отдельная.
    WidgetId
);

impl WidgetId {
    /// Создаёт `WidgetId` с проверкой формата `namespace.kebab`.
    ///
    /// Отдельно от [`WidgetId::new`], потому что точка допустима только здесь.
    pub fn parse(value: impl Into<String>) -> Result<Self, IdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdError::Empty);
        }
        if value.len() > MAX_ID_LEN {
            return Err(IdError::TooLong {
                len: value.len(),
                max: MAX_ID_LEN,
            });
        }
        let (namespace, name) = match value.split_once('.') {
            Some(parts) => parts,
            None => {
                return Err(IdError::InvalidChar { ch: '?', index: 0 });
            }
        };
        // Пространство имён и имя проверяются тем же правилом.
        is_valid_id(namespace)?;
        is_valid_id(name)?;
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn accepts_typical_ids() {
        assert!(ElementId::new("el_1").is_ok());
        assert!(ElementId::new("a").is_ok());
        assert!(ElementId::new("clock-2026_10").is_ok());
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(ElementId::new(""), Err(IdError::Empty));
    }

    #[test]
    fn rejects_uppercase_and_spaces() {
        // Формат намеренно строгий: id попадает в имена файлов и URL.
        assert!(matches!(
            ElementId::new("El1"),
            Err(IdError::InvalidChar { ch: 'E', .. })
        ));
        assert!(matches!(
            ElementId::new("el 1"),
            Err(IdError::InvalidChar { ch: ' ', index: 2 })
        ));
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(MAX_ID_LEN + 1);
        assert!(matches!(
            ElementId::new(long),
            Err(IdError::TooLong { len: 65, max: 64 })
        ));
    }

    #[test]
    fn widget_id_requires_namespace() {
        assert!(WidgetId::parse("clock").is_err());
        assert!(WidgetId::parse("core.clock").is_ok());
        assert!(WidgetId::parse("core.clock.extra").is_err());
        assert!(WidgetId::parse("core.Clock").is_err());
    }

    #[test]
    fn serialises_as_plain_string() {
        let id = MonitorId::new("m1").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"m1\"");
    }

    #[test]
    fn deserialisation_validates_and_rejects_garbage() {
        // Порча в БД не должна проходить молча.
        let err = serde_json::from_str::<ElementId>("\"BAD ID\"");
        assert!(err.is_err(), "deserialisation must validate");
    }

    proptest! {
        /// Любая строка из допустимого алфавита проходит, и round-trip через
        /// JSON сохраняет значение байт в байт.
        #[test]
        fn roundtrip_of_valid_ids(s in "[a-z0-9_-]{1,64}") {
            let id = ElementId::new(s.clone()).expect("generated id must be valid");
            let json = serde_json::to_string(&id).unwrap();
            let back: ElementId = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(id.as_str(), s.as_str());
            prop_assert_eq!(back, id);
        }

        /// Любой идентификатор короче лимита и с точкой не проходит основную
        /// валидацию — это гарантия того, что точка осмысленна только в WidgetId.
        #[test]
        fn dot_is_rejected_by_plain_ids(s in "[a-z]{1,10}") {
            let with_dot = format!("{s}.{s}");
            prop_assert!(ElementId::new(with_dot).is_err());
        }
    }
}
