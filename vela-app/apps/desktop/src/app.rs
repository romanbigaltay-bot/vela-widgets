//! Сборка Tauri и цикл событий.
//!
//! Инициализация живёт в `RunEvent::Ready`, а не в `.setup`: ошибку создания
//! окна или трея нужно залогировать и вернуть код 1, а не позволить Tauri
//! запаниковать с невнятным сообщением.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use anyhow::Context;
use tauri::{AppHandle, Manager, Wry};

use crate::cli::CliOptions;
use crate::commands;
use crate::exit_code::ExitCodeVela;
use crate::state::{self, AppState};
use crate::surfaces::{self, SurfaceSpec};
use crate::tray;

/// Запускает приложение и возвращает код завершения.
pub(crate) fn run(options: CliOptions) -> ExitCodeVela {
    let exit_code = Arc::new(AtomicU8::new(ExitCodeVela::Ok.code()));

    match run_event_loop(options, Arc::clone(&exit_code)) {
        Ok(()) => ExitCodeVela::from_code(exit_code.load(Ordering::Relaxed)),
        Err(err) => {
            tracing::error!(error = %err, "не удалось запустить Tauri");
            ExitCodeVela::InitFailed
        }
    }
}

/// Собирает приложение и крутит цикл событий до выхода.
fn run_event_loop(options: CliOptions, exit_code: Arc<AtomicU8>) -> anyhow::Result<()> {
    let builder = tauri::Builder::default()
        // Плагин обязан быть первым: он перехватывает запуск до создания окон.
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            tracing::info!(
                args = ?argv,
                cwd = %cwd,
                "второй экземпляр: активирую первый"
            );
            tray::show_ui(app);
        }))
        .manage(AppState::new(options))
        .invoke_handler(tauri::generate_handler![commands::window_surfaces_status])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if surfaces::is_ui_label(window.label()) {
                    // Alt+F4 не завершает приложение: поверхность прячется,
                    // процесс остаётся в трее. Выход — только через меню.
                    api.prevent_close();
                    tray::set_ui_visible(window.app_handle(), false);
                }
            }
        });

    let app = builder.build(tauri::generate_context!())?;

    app.run(move |app_handle, event| match event {
        tauri::RunEvent::Ready => {
            if let Err(err) = initialize(app_handle) {
                tracing::error!(error = %err, "инициализация приложения не удалась");
                exit_code.store(ExitCodeVela::InitFailed.code(), Ordering::Relaxed);
                app_handle.exit(i32::from(ExitCodeVela::InitFailed.code()));
            }
        }
        // Пользователь закрыл последнее окно: приложение живёт в трее.
        // `code: Some(_)` — это `app.exit()`; его отменять нельзя.
        tauri::RunEvent::ExitRequested {
            code: None, api, ..
        } => {
            let state = app_handle.state::<AppState>();
            if !state.quitting.load(Ordering::Relaxed) {
                api.prevent_exit();
            }
        }
        tauri::RunEvent::Exit => {
            // Закрываем окна до остановки рантайма: иначе `Drop` попытается
            // уничтожить их уже после того, как цикл событий умер.
            let state = app_handle.state::<AppState>();
            state::lock(&state.surfaces).destroy_all();
            tracing::info!("цикл событий завершён");
        }
        _ => {}
    });

    Ok(())
}

/// Создаёт поверхность UI и трей после готовности цикла событий.
fn initialize(app: &AppHandle<Wry>) -> anyhow::Result<()> {
    let spec = SurfaceSpec::ui_primary().context("некорректный идентификатор монитора")?;

    let statuses = {
        let state = app.state::<AppState>();
        let mut manager = state::lock(&state.surfaces);
        manager.ensure(app, spec)?;
        manager.status()
    };
    surfaces::emit_status(app, statuses);

    tray::create(app)?;
    tray::sync_check(app);

    // Второй экземпляр мог попросить показ до создания окна.
    let state = app.state::<AppState>();
    if state.pending_show.swap(false, Ordering::SeqCst) {
        tray::set_ui_visible(app, true);
    }

    tracing::info!(options = ?state.options, "приложение готово");
    Ok(())
}
