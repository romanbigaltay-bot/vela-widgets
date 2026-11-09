//! Реестр поверхностей и создание окон.
//!
//! # Почему окна создаются кодом
//!
//! `tauri.conf.json` не умеет выставлять `WS_EX_NOACTIVATE`, `WS_EX_TOOLWINDOW`
//! и `WS_EX_TRANSPARENT` до первого показа. Окно, созданное декларативно,
//! успевает мигнуть в панели задач и получить фокус — для фонового слоя это
//! недопустимо. Поэтому все поверхности создаются здесь: сначала скрытыми,
//! затем получают стили, и только потом могут быть показаны.
//!
//! # Почему хендлы не уходят в IPC
//!
//! `HWND` — это указатель процесса. В TypeScript он бесполезен, а как
//! публичный контракт опасен: UI начал бы зависеть от Win32. Наружу отдаётся
//! [`SurfaceStatus`] — тип, монитор, состояние и логический размер.

use std::collections::HashMap;
use std::fmt;

use tauri::{AppHandle, Emitter, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Wry};
use vela_ipc::{IdError, MonitorId, SurfaceKind, SurfaceState, SurfaceStatus};
use vela_platform::win::{self, ExStyle};

/// Префикс метки окна поверхности UI.
///
/// По метке обработчик `CloseRequested` отличает поверхность B от будущих
/// окон настроек: Alt+F4 на B скрывает её, а не завершает приложение.
pub(crate) const UI_WINDOW_PREFIX: &str = "vela-ui-";

/// Описание поверхности, которую нужно создать.
///
/// Поля повторяют контракт T-0201. `parent_to_workerw` пока не поддерживается:
/// привязка к WorkerW появится в T-0202, и молча игнорировать флаг нельзя —
/// поверхность оказалась бы не там, где ожидает вызывающий.
#[derive(Debug, Clone)]
pub(crate) struct SurfaceSpec {
    /// Тип поверхности.
    pub(crate) kind: SurfaceKind,
    /// Монитор, на котором живёт поверхность.
    pub(crate) monitor: MonitorId,
    /// Прозрачный фон окна.
    pub(crate) transparent: bool,
    /// Клики проходят сквозь окно.
    pub(crate) click_through: bool,
    /// Окно не активируется пользователем.
    pub(crate) no_activate: bool,
    /// Окно поверх всех остальных.
    pub(crate) top_most: bool,
    /// Привязать окно к WorkerW (T-0202).
    pub(crate) parent_to_workerw: bool,
    /// Ширина в логических пикселях.
    pub(crate) width: u32,
    /// Высота в логических пикселях.
    pub(crate) height: u32,
}

impl SurfaceSpec {
    /// Поверхность UI на основном мониторе — единственная в T-0201.
    ///
    /// `transparent: false` осознанно: прозрачность WebView2 и hit-regions —
    /// объём T-0203. Сейчас окно должно просто работать и не мигать.
    pub(crate) fn ui_primary() -> Result<Self, IdError> {
        Ok(Self {
            kind: SurfaceKind::Ui,
            monitor: MonitorId::new("primary")?,
            transparent: false,
            click_through: false,
            no_activate: true,
            top_most: false,
            parent_to_workerw: false,
            width: 800,
            height: 600,
        })
    }
}

/// Идентификатор поверхности внутри менеджера.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct SurfaceId(u64);

/// Ошибки создания и изменения поверхностей.
#[derive(Debug)]
pub(crate) enum SurfaceError {
    /// Tauri не смог создать окно или вызвать его метод.
    Tauri(tauri::Error),
    /// Win32 вернул ошибку.
    Platform(vela_platform::PlatformError),
    /// Запрошена возможность, которой ещё нет.
    Unsupported(&'static str),
    /// Поверхность не найдена: окно уже уничтожено.
    WindowGone,
}

impl fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tauri(err) => write!(f, "tauri: {err}"),
            Self::Platform(err) => write!(f, "platform: {err}"),
            Self::Unsupported(what) => write!(f, "unsupported: {what}"),
            Self::WindowGone => f.write_str("surface window is gone"),
        }
    }
}

impl std::error::Error for SurfaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Tauri(err) => Some(err),
            Self::Platform(err) => Some(err),
            Self::Unsupported(_) | Self::WindowGone => None,
        }
    }
}

impl From<tauri::Error> for SurfaceError {
    fn from(err: tauri::Error) -> Self {
        Self::Tauri(err)
    }
}

impl From<vela_platform::PlatformError> for SurfaceError {
    fn from(err: vela_platform::PlatformError) -> Self {
        Self::Platform(err)
    }
}

/// Одна созданная поверхность.
struct SurfaceHandle {
    id: SurfaceId,
    kind: SurfaceKind,
    monitor: MonitorId,
    width: u32,
    height: u32,
    window: WebviewWindow<Wry>,
    destroyed: bool,
}

impl SurfaceHandle {
    /// Снимок для UI. Видимость читается из окна, а не хранится копией:
    /// Alt+F4, скрытие из трея и будущие анимации не должны расходиться.
    fn status(&self) -> SurfaceStatus {
        let visible = self.window.is_visible().unwrap_or(false);
        SurfaceStatus {
            kind: self.kind,
            monitor: self.monitor.clone(),
            state: if visible {
                SurfaceState::Visible
            } else {
                SurfaceState::Hidden
            },
            width: self.width,
            height: self.height,
        }
    }

    /// Уничтожает окно ровно один раз.
    fn destroy(&mut self) -> Result<(), SurfaceError> {
        if self.destroyed {
            return Ok(());
        }
        self.destroyed = true;
        self.window.destroy()?;
        Ok(())
    }
}

impl Drop for SurfaceHandle {
    fn drop(&mut self) {
        if self.destroyed {
            return;
        }
        // Best-effort: при выходе цикл Tauri может быть уже остановлен.
        if let Err(err) = self.window.destroy() {
            tracing::debug!(
                error = %err,
                kind = self.kind.as_str(),
                "не удалось уничтожить окно поверхности"
            );
        }
    }
}

/// Реестр поверхностей: единственная точка создания и уничтожения окон.
#[derive(Default)]
pub(crate) struct SurfaceManager {
    next_id: u64,
    surfaces: HashMap<SurfaceId, SurfaceHandle>,
}

impl SurfaceManager {
    /// Создаёт поверхность, если такой `(kind, monitor)` ещё нет.
    ///
    /// Повторный вызов возвращает существующий id — это важно для
    /// восстановления после перезапуска Explorer и для команд, которые
    /// не должны плодить окна.
    pub(crate) fn ensure(
        &mut self,
        app: &AppHandle<Wry>,
        spec: SurfaceSpec,
    ) -> Result<SurfaceId, SurfaceError> {
        if spec.parent_to_workerw {
            return Err(SurfaceError::Unsupported(
                "parent_to_workerw появится в T-0202",
            ));
        }

        if let Some(existing) = self
            .surfaces
            .values()
            .find(|handle| handle.kind == spec.kind && handle.monitor == spec.monitor)
        {
            return Ok(existing.id);
        }

        let window = create_window(app, &spec)?;
        let id = SurfaceId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);

        tracing::info!(
            kind = spec.kind.as_str(),
            monitor = %spec.monitor,
            width = spec.width,
            height = spec.height,
            "поверхность создана"
        );

        self.surfaces.insert(
            id,
            SurfaceHandle {
                id,
                kind: spec.kind,
                monitor: spec.monitor,
                width: spec.width,
                height: spec.height,
                window,
                destroyed: false,
            },
        );
        Ok(id)
    }

    /// Снимок всех поверхностей в стабильном порядке.
    pub(crate) fn status(&self) -> Vec<SurfaceStatus> {
        let mut statuses: Vec<SurfaceStatus> =
            self.surfaces.values().map(SurfaceHandle::status).collect();
        statuses.sort_by(|a, b| {
            a.kind
                .as_str()
                .cmp(b.kind.as_str())
                .then_with(|| a.monitor.as_str().cmp(b.monitor.as_str()))
        });
        statuses
    }

    /// Есть ли поверхность такого типа.
    pub(crate) fn has_kind(&self, kind: SurfaceKind) -> bool {
        self.surfaces.values().any(|handle| handle.kind == kind)
    }

    /// Видима ли поверхность такого типа.
    ///
    /// Если окон несколько, достаточно одного видимого: меню трея управляет
    /// типом, а не конкретным монитором.
    pub(crate) fn kind_visible(&self, kind: SurfaceKind) -> bool {
        self.surfaces
            .values()
            .any(|handle| handle.kind == kind && handle.window.is_visible().unwrap_or(false))
    }

    /// Показывает или скрывает все поверхности указанного типа.
    ///
    /// Меню трея управляет типом, а не конкретным монитором: если поверхностей
    /// несколько, пользователь ждёт, что переключатся все. Отсутствие хотя бы
    /// одной поверхности — ошибка: вызывающий не должен думать, что что-то
    /// произошло, когда не произошло ничего.
    pub(crate) fn set_kind_visible(
        &mut self,
        kind: SurfaceKind,
        visible: bool,
    ) -> Result<(), SurfaceError> {
        let mut found = false;

        for handle in self
            .surfaces
            .values_mut()
            .filter(|handle| handle.kind == kind)
        {
            found = true;
            if visible {
                handle.window.show()?;
                // Фокус — best-effort: WS_EX_NOACTIVATE вправе запретить
                // активацию. Молча проглотить ошибку нельзя, но и валить показ
                // из-за неё тоже: окно уже видно, и пользователь этого ждёт.
                if let Err(err) = handle.window.set_focus() {
                    tracing::debug!(error = %err, "поверхность показана без фокуса");
                }
            } else {
                handle.window.hide()?;
            }
        }

        if !found {
            return Err(SurfaceError::WindowGone);
        }

        tracing::debug!(kind = kind.as_str(), visible, "видимость поверхности");
        Ok(())
    }

    /// Переключает видимость и возвращает новое состояние.
    pub(crate) fn toggle_kind(&mut self, kind: SurfaceKind) -> Result<bool, SurfaceError> {
        let visible = self.kind_visible(kind);
        self.set_kind_visible(kind, !visible)?;
        Ok(!visible)
    }

    /// Уничтожает поверхность по идентификатору.
    pub(crate) fn destroy(&mut self, id: SurfaceId) -> Result<(), SurfaceError> {
        let mut handle = self.surfaces.remove(&id).ok_or(SurfaceError::WindowGone)?;
        handle.destroy()
    }

    /// Уничтожает все поверхности (выход из приложения).
    pub(crate) fn destroy_all(&mut self) {
        let ids: Vec<SurfaceId> = self.surfaces.keys().copied().collect();
        for id in ids {
            if let Err(err) = self.destroy(id) {
                tracing::debug!(error = %err, "не удалось уничтожить поверхность");
            }
        }
    }
}

/// Создаёт скрытое окно и доводит его до нужного Win32-состояния.
fn create_window(
    app: &AppHandle<Wry>,
    spec: &SurfaceSpec,
) -> Result<WebviewWindow<Wry>, SurfaceError> {
    let label = window_label(spec.kind, &spec.monitor);
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("Vela")
        .inner_size(f64::from(spec.width), f64::from(spec.height))
        .resizable(false)
        .decorations(false)
        .shadow(false)
        .transparent(spec.transparent)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
        .center()
        .build()?;

    apply_platform_style(&window, spec)?;
    Ok(window)
}

/// Выставляет расширенные стили до первого показа окна.
fn apply_platform_style(
    window: &WebviewWindow<Wry>,
    spec: &SurfaceSpec,
) -> Result<(), SurfaceError> {
    let raw = window.hwnd()?.0 as isize;

    // TOOLWINDOW обязателен всем поверхностям: они не должны появляться
    // в панели задач и Alt+Tab, даже когда видимы.
    let mut style = ExStyle::TOOLWINDOW;
    if spec.no_activate {
        style |= ExStyle::NOACTIVATE;
    }
    if spec.click_through {
        style |= ExStyle::TRANSPARENT;
    }
    win::add_ex_style(raw, style)?;

    if spec.top_most {
        win::set_topmost(raw, true)?;
    }

    tracing::debug!(
        kind = spec.kind.as_str(),
        ex_style = format_args!("{:#x}", win::ex_style(raw).bits()),
        "стили поверхности применены"
    );
    Ok(())
}

/// Метка окна Tauri: `vela-<kind>-<monitor>`.
fn window_label(kind: SurfaceKind, monitor: &MonitorId) -> String {
    format!("vela-{}-{}", kind.as_str(), monitor.as_str())
}

/// Относится ли метка к поверхности UI.
pub(crate) fn is_ui_label(label: &str) -> bool {
    label.starts_with(UI_WINDOW_PREFIX)
}

/// Отправляет UI снимок поверхностей.
///
/// Ошибка отправки не фатальна: событие информационное, а следующий вызов
/// `window.surfaces_status` всё равно вернёт актуальные данные.
pub(crate) fn emit_status(app: &AppHandle<Wry>, statuses: Vec<SurfaceStatus>) {
    if let Err(err) = app.emit("window.surfaces_changed", statuses) {
        tracing::debug!(error = %err, "не удалось отправить window.surfaces_changed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_spec_is_hidden_and_non_activating() {
        let spec = SurfaceSpec::ui_primary().expect("primary is a valid monitor id");
        assert_eq!(spec.kind, SurfaceKind::Ui);
        assert_eq!(spec.monitor.as_str(), "primary");
        assert!(spec.no_activate);
        assert!(!spec.click_through);
        assert!(!spec.top_most);
        assert!(!spec.parent_to_workerw);
        assert_eq!((spec.width, spec.height), (800, 600));
    }

    #[test]
    fn window_label_is_stable() {
        let monitor = MonitorId::new("primary").expect("valid monitor id");
        assert_eq!(window_label(SurfaceKind::Ui, &monitor), "vela-ui-primary");
        assert!(is_ui_label("vela-ui-primary"));
        assert!(!is_ui_label("vela-scene-primary"));
    }

    #[test]
    fn empty_manager_has_no_surfaces() {
        let manager = SurfaceManager::default();
        assert!(manager.status().is_empty());
        assert!(!manager.has_kind(SurfaceKind::Ui));
        assert!(!manager.kind_visible(SurfaceKind::Ui));
    }

    #[test]
    fn missing_surface_is_an_error_not_a_silent_noop() {
        let mut manager = SurfaceManager::default();
        assert!(matches!(
            manager.set_kind_visible(SurfaceKind::Ui, true),
            Err(SurfaceError::WindowGone)
        ));
        assert!(matches!(
            manager.toggle_kind(SurfaceKind::Ui),
            Err(SurfaceError::WindowGone)
        ));
    }
}
