//! Конверт ответа команды.
//!
//! # Почему не `Result<T, E>` напрямую
//!
//! Tauri сериализует `Result` как `{"Ok": ...}` / `{"Err": ...}`, что на стороне
//! TS превращается в нетипизированный union и требует ручных проверок в каждом
//! вызове. Конверт даёт ровно один стабильный формат на все команды:
//!
//! ```json
//! { "ok": true,  "data": { ... }, "error": null }
//! { "ok": false, "data": null,    "error": { "code": "busy", ... } }
//! ```
//!
//! Инвариант (проверяется тестами и типами): `ok == data.is_some() != error.is_some()`.
//! То есть успех всегда несёт данные, ошибка всегда несёт ошибку, и оба
//! одновременно невозможны.

use crate::errors::VelaError;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Ответ команды IPC.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct Envelope<T: TS> {
    /// Успешно ли выполнена команда.
    pub ok: bool,
    /// Данные при `ok == true`.
    pub data: Option<T>,
    /// Ошибка при `ok == false`.
    pub error: Option<VelaError>,
}

impl<T: TS> Envelope<T> {
    /// Успешный ответ с данными.
    pub fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    /// Ответ с ошибкой (данных нет).
    pub fn err(error: VelaError) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(error),
        }
    }

    /// Разворачивает конверт в `Result`, если данные нужны в Rust-коде
    /// (например, в тестах или при межкрейтовом вызове).
    pub fn into_result(self) -> Result<T, VelaError> {
        match (self.ok, self.data, self.error) {
            (true, Some(data), _) => Ok(data),
            (false, _, Some(error)) => Err(error),
            // Недостижимо при использовании конструкторов, но конверт может
            // прийти из внешнего JSON — поэтому не паникуем, а сообщаем.
            (true, None, _) => Err(VelaError::new(
                crate::errors::ErrorCode::Internal,
                "envelope is marked ok but carries no data",
            )),
            (false, _, None) => Err(VelaError::new(
                crate::errors::ErrorCode::Internal,
                "envelope is marked not-ok but carries no error",
            )),
        }
    }

    /// Проверяет инвариант конверта. Возвращает `true`, если состояние корректно.
    ///
    /// Используется в dev-валидации на стороне UI и в тестах: `ok` не должен
    /// противоречить наличию данных/ошибки.
    pub fn is_consistent(&self) -> bool {
        matches!(
            (self.ok, self.data.is_some(), self.error.is_some()),
            (true, true, false) | (false, false, true)
        )
    }
}

/// Конкретное инстанцирование [`Envelope`] для генерации TypeScript-типа.
///
/// `Envelope` — generic, а `ts-rs` умеет экспортировать только конкретные типы.
/// Этот алиас существует ровно для кодогенерации: он заставляет ts-rs создать
/// `Envelope.ts` с параметром типа, а UI подставляет в него реальные `T`.
///
/// В рантайме не используется — поэтому `#[allow(dead_code)]`.
#[allow(dead_code)]
pub type UnitEnvelope = Envelope<()>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCode;

    // `TS` нужен, чтобы использовать Payload в Envelope, но `export` — нет:
    // тестовый тип не должен попадать в продакшн-биндинги UI.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
    struct Payload {
        value: u32,
    }

    #[test]
    fn ok_envelope_carries_data() {
        let env = Envelope::ok(Payload { value: 7 });
        assert!(env.ok);
        assert!(env.is_consistent());
        assert_eq!(env.data, Some(Payload { value: 7 }));
        assert!(env.error.is_none());
    }

    #[test]
    fn err_envelope_carries_error() {
        let env: Envelope<Payload> = Envelope::err(VelaError::new(ErrorCode::Busy, "busy"));
        assert!(!env.ok);
        assert!(env.is_consistent());
        assert!(env.data.is_none());
        assert_eq!(env.error.unwrap().code, ErrorCode::Busy);
    }

    #[test]
    fn into_result_roundtrip() {
        let env = Envelope::ok(Payload { value: 1 });
        assert_eq!(env.into_result().unwrap(), Payload { value: 1 });

        let env: Envelope<Payload> = Envelope::err(VelaError::new(ErrorCode::NotFound, "x"));
        assert_eq!(env.into_result().unwrap_err().code, ErrorCode::NotFound);
    }

    #[test]
    fn inconsistent_envelope_is_detected_not_panicking() {
        // Такой конверт может прийти из чужого JSON; поведение должно быть
        // предсказуемым и без паники.
        let broken = Envelope::<Payload> {
            ok: true,
            data: None,
            error: None,
        };
        assert!(!broken.is_consistent());
        let err = broken.into_result().unwrap_err();
        assert_eq!(err.code, ErrorCode::Internal);
    }

    #[test]
    fn wire_shape_is_stable() {
        // Контракт с TS: три поля, ровно такие имена.
        let env = Envelope::ok(Payload { value: 1 });
        let json = serde_json::to_value(&env).unwrap();
        let obj = json.as_object().unwrap();
        let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["data", "error", "ok"]);
        assert_eq!(obj["ok"], serde_json::json!(true));
        assert_eq!(obj["error"], serde_json::Value::Null);
    }
}
