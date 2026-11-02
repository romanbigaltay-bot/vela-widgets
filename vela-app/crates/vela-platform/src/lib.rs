//! Платформенный слой Vela.
//!
//! На Windows здесь живут обёртки Win32 (окна, WorkerW, DPI, захват).
//! На других ОС крейт собирается, но любая платформенная операция возвращает
//! [`PlatformError::Unsupported`] — это позволяет гонять линты и тесты домена
//! на любой машине разработчика и в Linux-джобе CI.

// Тесты могут паниковать: `unwrap()` там — это утверждение об ожидаемом
// результате, а не обработка ошибки (см. clippy.toml, unwrap_used).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use thiserror::Error;

/// Идентификатор сборки платформенного слоя.
pub fn health() -> &'static str {
    "vela-platform"
}

/// Ошибки платформенного слоя.
///
/// Тексты намеренно короткие и машинно-читаемые: пользователю показывается
/// локализованная строка из UI, а не это сообщение.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlatformError {
    /// Операция недоступна на текущей ОС (не Windows).
    #[error("operation is not supported on this platform")]
    Unsupported,

    /// Windows недоступен: нет WorkerW, отключён сценарный слой, RDP-сессия.
    #[error("platform feature unavailable: {0}")]
    Unavailable(&'static str),

    /// Win32 вернул ошибку; код сохраняется для диагностики.
    #[error("win32 error {code}: {context}")]
    Win32 {
        /// Код из `GetLastError`.
        code: u32,
        /// Что именно делали.
        context: &'static str,
    },
}

/// Признак того, что сборка включает настоящий Win32-слой.
pub const IS_WINDOWS: bool = cfg!(windows);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_stable() {
        assert_eq!(health(), "vela-platform");
    }

    #[test]
    fn error_display_is_stable() {
        // Тексты ошибок попадают в логи и в отчёты диагностики — их формат
        // не должен меняться незаметно.
        assert_eq!(
            PlatformError::Unsupported.to_string(),
            "operation is not supported on this platform"
        );
        assert_eq!(
            PlatformError::Win32 {
                code: 5,
                context: "SetParent"
            }
            .to_string(),
            "win32 error 5: SetParent"
        );
    }
}
