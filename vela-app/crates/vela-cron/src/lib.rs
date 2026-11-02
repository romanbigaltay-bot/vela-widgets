//! Планировщик: парсинг cron-выражений и расчёт следующего срабатывания.
//!
//! В P1 крейт содержит только каркас: типы и проверку работоспособности.
//! Реализация парсера — T-0704/ST-0704-01.

// Тесты могут паниковать: `unwrap()` там — это утверждение об ожидаемом
// результате, а не обработка ошибки (см. clippy.toml, unwrap_used).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use thiserror::Error;

/// Идентификатор сборки планировщика.
pub fn health() -> &'static str {
    "vela-cron"
}

/// Ошибки разбора и расчёта расписания.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CronError {
    /// Выражение не соответствует формату из 5 полей.
    #[error("expected 5 fields, got {0}")]
    FieldCount(usize),

    /// Поле содержит недопустимый токен.
    #[error("invalid token `{token}` in field {field}")]
    InvalidToken {
        /// Имя поля: minute/hour/dom/month/dow.
        field: &'static str,
        /// Токен, который не удалось разобрать.
        token: String,
    },

    /// Комбинация синтаксически верна, но не встречается в календаре
    /// (например, 31 февраля).
    #[error("schedule never fires")]
    NeverFires,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_stable() {
        assert_eq!(health(), "vela-cron");
    }

    #[test]
    fn field_count_error_is_readable() {
        assert_eq!(
            CronError::FieldCount(3).to_string(),
            "expected 5 fields, got 3"
        );
    }
}
