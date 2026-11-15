//! Точка входа приложения Vela.
//!
//! В P1 здесь был только каркас. С T-0201 на Windows поднимается Tauri:
//! приложение живёт в трее, создаёт поверхность UI скрытой и показывает её
//! по клику. На других платформах остаётся headless-путь — он нужен, чтобы
//! workspace проходил `cargo test` в Linux CI без WebView2 и Win32.

// Тесты могут паниковать: `unwrap()` там — это утверждение об ожидаемом
// результате, а не обработка ошибки (см. clippy.toml, unwrap_used).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod bootstrap;
pub mod cli;
pub mod exit_code;

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod commands;
#[cfg(windows)]
mod state;
#[cfg(windows)]
mod surfaces;
#[cfg(windows)]
mod tray;

use std::process::ExitCode;

use cli::CliOptions;
use exit_code::ExitCodeVela;

/// Запускает приложение и возвращает код процесса.
///
/// Порядок важен: база открывается и мигрирует до появления окон. Если схему
/// не удалось привести в рабочее состояние, показывать холст с пустыми
/// виджетами бессмысленно — пользователь примет это за потерю данных.
pub fn run() -> ExitCode {
    init_tracing();
    log_health();

    let options = CliOptions::parse(std::env::args().skip(1));

    let prepared = match bootstrap::prepare() {
        Ok(prepared) => prepared,
        Err(code) => return code.into(),
    };

    #[cfg(windows)]
    let code = run_desktop(prepared, options);

    #[cfg(not(windows))]
    let code = {
        let _ = options;
        run_headless(prepared)
    };

    code.into()
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

/// Проверка связности workspace: если крейт не слинковался, приложение
/// обязано упасть на старте, а не в момент первого обращения.
fn log_health() {
    tracing::debug!(
        core = vela_core::health(),
        ipc = vela_ipc::health(),
        platform = vela_platform::health(),
        cron = vela_cron::health(),
        schema = vela_core::SCHEMA_VERSION,
        windows = vela_platform::IS_WINDOWS,
        "vela starting"
    );
}

/// Завершает приложение после остановки цикла Tauri.
///
/// База живёт вне `AppState`: окнам она пока не нужна, а закрывать её нужно
/// после того, как event loop отпустил окна и трей. Ошибка закрытия — это
/// ошибка инициализации: процесс не должен возвращать 0, если WAL не сброшен.
#[cfg(windows)]
fn run_desktop(prepared: bootstrap::Prepared, options: CliOptions) -> ExitCodeVela {
    let code = app::run(options);
    match prepared.shutdown() {
        Ok(()) => code,
        Err(err) => {
            tracing::error!(error = %err, "не удалось корректно закрыть базу");
            ExitCodeVela::InitFailed
        }
    }
}

/// Путь без Tauri: Linux CI и другие ОС.
///
/// Приложение не показывает окно, но проверяет базу и завершается штатно —
/// ровно то поведение, которое было в P1.
#[cfg(not(windows))]
fn run_headless(prepared: bootstrap::Prepared) -> ExitCodeVela {
    tracing::info!("Tauri доступен только на Windows: headless-режим");
    match prepared.shutdown() {
        Ok(()) => ExitCodeVela::Ok,
        Err(err) => {
            tracing::error!(error = %err, "не удалось корректно закрыть базу");
            ExitCodeVela::InitFailed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_stable() {
        // Значения кодов — публичный контракт для установщика и CI.
        assert_eq!(ExitCodeVela::Ok.code(), 0);
        assert_eq!(ExitCodeVela::InitFailed.code(), 1);
        assert_eq!(ExitCodeVela::SchemaTooNew.code(), 2);
    }

    #[test]
    fn health_is_stable() {
        assert_eq!(vela_core::health(), "vela-core");
        assert_eq!(vela_ipc::health(), "vela-ipc");
        assert_eq!(vela_platform::health(), "vela-platform");
        assert_eq!(vela_cron::health(), "vela-cron");
    }
}
