//! Коды возврата приложения.
//!
//! Фиксированы, потому что внешние сценарии (CI, установщик, автообновление)
//! различают «сломанную установку» и «сломанный запуск».

use std::process::ExitCode;

/// Коды возврата, на которые опираются тесты и установщик.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCodeVela {
    /// Штатное завершение.
    Ok = 0,
    /// Ошибка инициализации (логирование, конфигурация, окно, трей).
    InitFailed = 1,
    /// Схема БД новее, чем понимает бинарь: нужно обновить приложение.
    SchemaTooNew = 2,
}

impl ExitCodeVela {
    /// Числовой код процесса.
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Восстанавливает код из атомарного счётчика цикла событий.
    ///
    /// Неизвестное значение считается успехом: это может быть только ошибка
    /// в нашем коде, и превращать её в «сломанную установку» нельзя.
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::InitFailed,
            2 => Self::SchemaTooNew,
            _ => Self::Ok,
        }
    }
}

impl From<ExitCodeVela> for ExitCode {
    fn from(code: ExitCodeVela) -> Self {
        Self::from(code.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(ExitCodeVela::Ok.code(), 0);
        assert_eq!(ExitCodeVela::InitFailed.code(), 1);
        assert_eq!(ExitCodeVela::SchemaTooNew.code(), 2);
    }

    #[test]
    fn from_code_is_total() {
        assert_eq!(ExitCodeVela::from_code(0), ExitCodeVela::Ok);
        assert_eq!(ExitCodeVela::from_code(1), ExitCodeVela::InitFailed);
        assert_eq!(ExitCodeVela::from_code(2), ExitCodeVela::SchemaTooNew);
        assert_eq!(ExitCodeVela::from_code(255), ExitCodeVela::Ok);
    }
}
