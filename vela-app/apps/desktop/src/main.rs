//! Точка входа приложения Vela.
//!
//! В P1 здесь только каркас: инициализация логирования, проверка связности
//! крейтов и корректный код возврата. Окна появляются в T-0201 (ST-0201-01),
//! поэтому `tauri::Builder` в этом файле ещё отсутствует — это осознанно,
//! так гейт G1 не зависит от графического стека и проверяется в любой CI.

// Тесты могут паниковать: `unwrap()` там — это утверждение об ожидаемом
// результате, а не обработка ошибки (см. clippy.toml, unwrap_used).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

use std::process::ExitCode;

/// Коды возврата, на которые опираются тесты и установщик.
///
/// Фиксированы, потому что внешние сценарии (CI, установщик, автообновление)
/// различают «сломанную установку» и «сломанный запуск».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCodeVela {
    /// Штатное завершение.
    Ok = 0,
    /// Ошибка инициализации (логирование, конфигурация).
    InitFailed = 1,
    /// Схема БД новее, чем понимает бинарь: нужно обновить приложение.
    SchemaTooNew = 2,
}

fn main() -> ExitCode {
    init_tracing();

    // Проверка связности workspace: если какой-то крейт не слинковался,
    // приложение обязано упасть на старте, а не в момент первого обращения.
    let parts = (
        vela_core::health(),
        vela_ipc::health(),
        vela_platform::health(),
        vela_cron::health(),
    );
    tracing::debug!(
        core = parts.0,
        ipc = parts.1,
        platform = parts.2,
        cron = parts.3,
        schema = vela_core::SCHEMA_VERSION,
        windows = vela_platform::IS_WINDOWS,
        "vela starting"
    );

    match run() {
        Ok(code) => code,
        Err(err) => {
            tracing::error!(error = %err, "fatal");
            ExitCode::from(ExitCodeVela::InitFailed as u8)
        }
    }
}

/// Логирование пишется в stderr до появления файлового лога (T-0805).
///
/// `RUST_LOG` уважается, но по умолчанию уровень `info`, чтобы не заливать
/// консоль при запуске из проводника.
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}

fn run() -> anyhow::Result<ExitCode> {
    // База открывается до появления окон: если схему не удалось привести в
    // рабочее состояние, показывать пользователю холст с пустыми виджетами
    // бессмысленно — он выглядит как потеря данных, а не как ошибка.
    let mut db = match vela_core::Db::open_default() {
        Ok(db) => db,
        Err(err) => {
            tracing::error!(error = %err, "не удалось открыть базу данных");
            return Ok(ExitCode::from(ExitCodeVela::InitFailed as u8));
        }
    };

    // Восстановление после порчи — не ошибка запуска: приложение обязано
    // работать, а пользователь узнать, что старый файл сохранён рядом.
    if let vela_core::OpenOutcome::Recovered { backup } = db.outcome() {
        tracing::warn!(
            backup = %backup.display(),
            "база была повреждена: файл сохранён, создана новая"
        );
    }

    match vela_core::migrate(&db) {
        Ok(vela_core::MigrateOutcome::Applied { from, to, backup }) => {
            tracing::info!(from, to, backup = ?backup, "схема обновлена");
        }
        // UpToDate намеренно не логируется: библиотека уже сообщила это
        // на уровне debug, а повтор того же сообщения из бинаря только
        // удваивает строки в файле лога.
        Ok(vela_core::MigrateOutcome::UpToDate { .. }) => {}
        Err(vela_core::CoreError::SchemaTooNew { found, supported }) => {
            // Приложение старше базы: это результат отката на предыдущую
            // версию. Понижать схему нельзя, поэтому сообщаем и выходим
            // отдельным кодом, чтобы обновлятор это увидел.
            tracing::error!(found, supported, "база создана более новой версией");
            let _ = db.shutdown();
            return Ok(ExitCode::from(ExitCodeVela::SchemaTooNew as u8));
        }
        Err(err) => {
            tracing::error!(error = %err, "не удалось применить миграции");
            let _ = db.shutdown();
            return Ok(ExitCode::from(ExitCodeVela::InitFailed as u8));
        }
    }

    // Здесь в T-0201 появится запуск Tauri и менеджера поверхностей.
    // Пока — успешный выход, чтобы `cargo run -p vela` возвращал 0 в P1.
    db.shutdown()?;
    Ok(ExitCode::from(ExitCodeVela::Ok as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_stable() {
        // Значения кодов — публичный контракт для установщика и CI.
        assert_eq!(ExitCodeVela::Ok as u8, 0);
        assert_eq!(ExitCodeVela::InitFailed as u8, 1);
        assert_eq!(ExitCodeVela::SchemaTooNew as u8, 2);
    }

    #[test]
    fn run_succeeds_in_skeleton() {
        assert!(run().is_ok());
    }
}
