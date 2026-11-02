//! Коды ошибок и структура ошибки IPC.
//!
//! # Зачем отдельный код, а не только сообщение
//!
//! UI принимает решения по коду: `PermissionDenied` — показать «нет доступа» и
//! не предлагать повтор; `PlatformUnavailable` — предложить упрощённый режим;
//! `Busy` — подождать и повторить самому. Текст сообщения локализуется и может
//! меняться, поэтому никогда не используется как признак.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Машиночитаемый код ошибки.
///
/// Набор закрыт намеренно: добавление варианта — несовместимое изменение,
/// требующее повышения `IPC_VERSION`, потому что UI обрабатывает неизвестный
/// код как `Internal` и показывает общий текст.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Вызывающий не имеет права на операцию (см. `09-widget-spec.md` §4).
    PermissionDenied,
    /// Запрошенный объект не существует.
    NotFound,
    /// Возможность недоступна на этой машине: нет WorkerW, RDP, нет GPU,
    /// отсутствует языковой пакет OCR.
    PlatformUnavailable,
    /// Ресурс занят: идёт другой захват, элемент редактируется в другом окне.
    Busy,
    /// Входные данные не прошли валидацию.
    InvalidInput,
    /// Внутренняя ошибка: баг или необработанное состояние.
    Internal,
}

impl ErrorCode {
    /// Стоит ли предлагать пользователю повтор.
    ///
    /// Используется UI, чтобы не показывать «повторить» там, где повтор
    /// гарантированно даст тот же результат.
    pub fn is_retryable(self) -> bool {
        matches!(self, ErrorCode::Busy | ErrorCode::Internal)
    }

    /// Стабильное строковое имя для логов (не зависит от сериализации,
    /// чтобы лог не менялся при смене `rename_all`).
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::PermissionDenied => "permission_denied",
            ErrorCode::NotFound => "not_found",
            ErrorCode::PlatformUnavailable => "platform_unavailable",
            ErrorCode::Busy => "busy",
            ErrorCode::InvalidInput => "invalid_input",
            ErrorCode::Internal => "internal",
        }
    }
}

/// Ошибка, возвращаемая командой IPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct VelaError {
    /// Что именно пошло не так — по этому полю принимает решения UI.
    pub code: ErrorCode,
    /// Диагностический текст. Не для показа пользователю без локализации.
    pub message: String,
    /// Можно ли повторять операцию. Дублирует `code.is_retryable()`,
    /// но передаётся явно: источник ошибки (Rust или плагин) может знать
    /// больше, чем позволяет код.
    pub retryable: bool,
}

impl VelaError {
    /// Создаёт ошибку с автоматическим `retryable` по коду.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: code.is_retryable(),
        }
    }

    /// Ошибка «нет такого объекта».
    pub fn not_found(what: impl AsRef<str>, id: impl std::fmt::Display) -> Self {
        Self::new(
            ErrorCode::NotFound,
            format!("{} `{}` does not exist", what.as_ref(), id),
        )
    }

    /// Ошибка валидации.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    /// Возможность недоступна на платформе.
    pub fn unavailable(what: impl Into<String>) -> Self {
        Self::new(ErrorCode::PlatformUnavailable, what)
    }
}

impl std::fmt::Display for VelaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for VelaError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_matches_code() {
        assert!(ErrorCode::Busy.is_retryable());
        assert!(ErrorCode::Internal.is_retryable());
        assert!(!ErrorCode::PermissionDenied.is_retryable());
        assert!(!ErrorCode::InvalidInput.is_retryable());
        assert!(!ErrorCode::NotFound.is_retryable());
        assert!(!ErrorCode::PlatformUnavailable.is_retryable());
    }

    #[test]
    fn new_derives_retryable_from_code() {
        let busy = VelaError::new(ErrorCode::Busy, "capture in progress");
        assert!(busy.retryable);
        let denied = VelaError::new(ErrorCode::PermissionDenied, "no net:http");
        assert!(!denied.retryable);
    }

    #[test]
    fn wire_format_is_snake_case() {
        // Контракт с TS: ровно эти строки. Смена = несовместимое изменение.
        let json = serde_json::to_string(&ErrorCode::PermissionDenied).unwrap();
        assert_eq!(json, "\"permission_denied\"");
        let json = serde_json::to_string(&ErrorCode::PlatformUnavailable).unwrap();
        assert_eq!(json, "\"platform_unavailable\"");
    }

    #[test]
    fn as_str_matches_wire_format() {
        // Логи и провод не должны разъезжаться: одно и то же имя.
        for code in [
            ErrorCode::PermissionDenied,
            ErrorCode::NotFound,
            ErrorCode::PlatformUnavailable,
            ErrorCode::Busy,
            ErrorCode::InvalidInput,
            ErrorCode::Internal,
        ] {
            let wire = serde_json::to_string(&code).unwrap();
            assert_eq!(wire, format!("\"{}\"", code.as_str()));
        }
    }

    #[test]
    fn display_is_log_friendly() {
        let e = VelaError::not_found("element", "el_1");
        assert_eq!(e.to_string(), "not_found: element `el_1` does not exist");
    }
}
