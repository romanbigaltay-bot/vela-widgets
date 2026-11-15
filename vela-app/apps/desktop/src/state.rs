//! Состояние приложения, разделяемое командами IPC и циклом событий.

use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::menu::CheckMenuItem;
use tauri::tray::TrayIcon;
use tauri::Wry;

use crate::cli::CliOptions;
use crate::surfaces::SurfaceManager;

/// Ресурсы трея, которые обязаны жить, пока живёт приложение.
///
/// `TrayIcon` удаляется при `Drop`, а `CheckMenuItem` — единственный способ
/// показать пользователю реальное состояние поверхности. Оба хранятся здесь,
/// а не в замыканиях: замыкания не дают доступа к чекбоксу из других событий.
pub(crate) struct TrayState {
    /// Живёт только ради владения: `TrayIcon` удаляет иконку при `Drop`.
    pub(crate) _icon: TrayIcon<Wry>,
    pub(crate) show_ui: CheckMenuItem<Wry>,
}

/// Общее состояние приложения.
pub(crate) struct AppState {
    /// Реестр поверхностей. `Mutex`, потому что Tauri требует `Sync`, а
    /// менеджер изменяется из команд и событий окна.
    pub(crate) surfaces: Mutex<SurfaceManager>,
    /// Трей появляется после `RunEvent::Ready`, поэтому `Option`.
    pub(crate) tray: Mutex<Option<TrayState>>,
    /// Флаги запуска; потребители появятся в следующих задачах P2.
    pub(crate) options: CliOptions,
    /// Второй экземпляр попросил показать UI до готовности поверхности.
    pub(crate) pending_show: AtomicBool,
    /// Пользователь выбрал «Выход»: `ExitRequested` больше не отменяется.
    pub(crate) quitting: AtomicBool,
}

impl AppState {
    /// Создаёт пустое состояние: окна и трей появятся в `RunEvent::Ready`.
    pub(crate) fn new(options: CliOptions) -> Self {
        Self {
            surfaces: Mutex::new(SurfaceManager::default()),
            tray: Mutex::new(None),
            options,
            pending_show: AtomicBool::new(false),
            quitting: AtomicBool::new(false),
        }
    }
}

/// Берёт блокировку, переживая отравление мьютекса.
///
/// Паника в одном обработчике не должна превращать приложение в кирпич:
/// данные — это реестр окон, а не бухгалтерия, и продолжить с последним
/// согласованным состоянием безопаснее, чем упасть на каждом следующем клике.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
