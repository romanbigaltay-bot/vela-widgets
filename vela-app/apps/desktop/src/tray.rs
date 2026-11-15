//! Трей: иконка, меню и переключение поверхности UI.
//!
//! Меню сознательно короткое. Пункты «сцена», «режим», «профиль» появятся
//! вместе с задачами, которые ими управляют (T-0202…T-0204): пункт, который
//! ничего не делает, хуже отсутствующего — пользователь считает его сломанным.

use std::sync::atomic::Ordering;

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};
use vela_ipc::SurfaceKind;

use crate::exit_code::ExitCodeVela;
use crate::state::{lock, AppState, TrayState};
use crate::surfaces;

/// Создаёт иконку трея и меню.
///
/// Вызывается после `RunEvent::Ready`: до готовности цикла событий Tauri не
/// может создать `TrayIcon`, а `CheckMenuItem` не к чему привязать.
pub(crate) fn create(app: &AppHandle<Wry>) -> tauri::Result<()> {
    let show_ui = CheckMenuItem::with_id(
        app,
        "toggle-ui",
        "Показать виджеты",
        true,
        false,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_ui, &quit_item])?;

    // На светлой панели задач нужен тёмный глиф, на тёмной — светлый.
    let icon = if vela_platform::win::system_uses_light_theme() {
        Image::from_bytes(include_bytes!("../icons/tray-light.png"))?
    } else {
        Image::from_bytes(include_bytes!("../icons/tray-dark.png"))?
    };

    let icon = TrayIconBuilder::with_id("vela-tray")
        .icon(icon)
        .tooltip("Vela")
        .menu(&menu)
        // Левый клик — переключение виджетов, правый — меню. Иначе меню
        // открывалось бы вместо показа, и до виджетов было бы два клика.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle-ui" => toggle_ui(app),
            "quit" => quit(app),
            other => tracing::debug!(id = other, "неизвестный пункт меню трея"),
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_ui(tray.app_handle());
            }
        })
        .build(app)?;

    let state = app.state::<AppState>();
    lock(&state.tray).replace(TrayState {
        _icon: icon,
        show_ui,
    });
    tracing::info!("трей создан");
    Ok(())
}

/// Показывает UI, если поверхность уже существует.
///
/// Второй экземпляр может подключиться раньше, чем `RunEvent::Ready` создаст
/// окно. В этом случае запрос запоминается и выполняется после инициализации.
pub(crate) fn show_ui(app: &AppHandle<Wry>) {
    let state = app.state::<AppState>();
    let exists = lock(&state.surfaces).has_kind(SurfaceKind::Ui);
    if !exists {
        state.pending_show.store(true, Ordering::SeqCst);
        tracing::info!("поверхность UI ещё не создана: показ отложен");
        return;
    }
    set_ui_visible(app, true);
}

/// Показывает или скрывает поверхность UI.
pub(crate) fn set_ui_visible(app: &AppHandle<Wry>, visible: bool) {
    update_visibility(app, |manager| {
        manager.set_kind_visible(SurfaceKind::Ui, visible)
    });
}

/// Переключает видимость поверхности UI.
pub(crate) fn toggle_ui(app: &AppHandle<Wry>) {
    update_visibility(app, |manager| {
        manager.toggle_kind(SurfaceKind::Ui).map(|_| ())
    });
}

/// Применяет изменение видимости, отправляет событие и синхронизирует чекбокс.
fn update_visibility<F>(app: &AppHandle<Wry>, change: F)
where
    F: FnOnce(&mut surfaces::SurfaceManager) -> Result<(), surfaces::SurfaceError>,
{
    let statuses = {
        let state = app.state::<AppState>();
        let mut manager = lock(&state.surfaces);
        match change(&mut manager) {
            Ok(()) => manager.status(),
            Err(err) => {
                tracing::error!(error = %err, "не удалось изменить видимость поверхности");
                return;
            }
        }
    };

    surfaces::emit_status(app, statuses);
    sync_check(app);
}

/// Приводит чекбокс меню к реальной видимости поверхности.
///
/// Меню Tauri переключает `CheckMenuItem` само до вызова обработчика, поэтому
/// после каждого изменения состояние всё равно перезаписывается из менеджера.
/// Это единственный способ не получить галочку, которая врёт.
pub(crate) fn sync_check(app: &AppHandle<Wry>) {
    let state = app.state::<AppState>();
    let visible = lock(&state.surfaces).kind_visible(SurfaceKind::Ui);
    let tray = lock(&state.tray);
    let Some(tray) = tray.as_ref() else {
        return;
    };
    if let Err(err) = tray.show_ui.set_checked(visible) {
        tracing::warn!(error = %err, "не удалось обновить чекбокс трея");
    }
}

/// Штатный выход. Единственный путь, который завершает процесс.
pub(crate) fn quit(app: &AppHandle<Wry>) {
    let state = app.state::<AppState>();
    state.quitting.store(true, Ordering::SeqCst);
    tracing::info!("выход через трей");
    app.exit(i32::from(ExitCodeVela::Ok.code()));
}
