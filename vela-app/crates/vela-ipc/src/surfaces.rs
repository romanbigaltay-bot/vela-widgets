//! Контракты поверхностей окна.
//!
//! Поверхность — это окно, которое ядро создаёт кодом, а не `tauri.conf.json`.
//! UI видит только описание: тип, монитор, состояние и размер. Нативные хендлы
//! (`HWND`) через IPC не передаются: они бессмысленны в TypeScript и опасны
//! как публичный контракт.
//!
//! # Почему состояние — отдельное перечисление
//!
//! `visible: bool` было бы достаточно сегодня, но завтра появится «создаётся»
//! и «упала». Строковый union в TS позволяет добавлять состояния без смены
//! формы ответа; `bool` пришлось бы заменять несовместимо.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ids::MonitorId;

/// Тип поверхности.
///
/// * `Scene` — слой под иконками рабочего стола (WorkerW).
/// * `Ui` — слой виджетов над иконками.
/// * `Overlay` — верхний слой для палитры, аннотаций и настроек.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceKind {
    /// Сцена под иконками.
    Scene,
    /// Виджеты и основной интерфейс.
    Ui,
    /// Оверлей поверх всех окон.
    Overlay,
}

impl SurfaceKind {
    /// Строковое имя для логов и меток окон.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scene => "scene",
            Self::Ui => "ui",
            Self::Overlay => "overlay",
        }
    }
}

/// Текущее состояние поверхности.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceState {
    /// Окно создано, но не показано.
    Hidden,
    /// Окно видно пользователю.
    Visible,
}

/// Снимок одной поверхности для UI.
///
/// Размеры — логические пиксели, те же, что передаются в `WebviewWindowBuilder`.
/// Физический размер зависит от DPI монитора, и смешивать их в одном поле
/// означало бы показывать пользователю разные числа на разных экранах.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SurfaceStatus {
    /// Тип поверхности.
    pub kind: SurfaceKind,
    /// Монитор, на котором живёт поверхность.
    pub monitor: MonitorId,
    /// Видима ли поверхность сейчас.
    pub state: SurfaceState,
    /// Ширина в логических пикселях.
    pub width: u32,
    /// Высота в логических пикселях.
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primary() -> MonitorId {
        MonitorId::new("primary").expect("валидный монитор")
    }

    #[test]
    fn kind_wire_format_is_snake_case() {
        // Эти строки уходят в TypeScript и в логи. Смена = несовместимое
        // изменение контракта, поэтому фиксируем их тестом.
        assert_eq!(
            serde_json::to_string(&SurfaceKind::Scene).unwrap(),
            "\"scene\""
        );
        assert_eq!(serde_json::to_string(&SurfaceKind::Ui).unwrap(), "\"ui\"");
        assert_eq!(
            serde_json::to_string(&SurfaceKind::Overlay).unwrap(),
            "\"overlay\""
        );
    }

    #[test]
    fn state_wire_format_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&SurfaceState::Hidden).unwrap(),
            "\"hidden\""
        );
        assert_eq!(
            serde_json::to_string(&SurfaceState::Visible).unwrap(),
            "\"visible\""
        );
    }

    #[test]
    fn status_round_trips_through_json() {
        let status = SurfaceStatus {
            kind: SurfaceKind::Ui,
            monitor: primary(),
            state: SurfaceState::Visible,
            width: 800,
            height: 600,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"ui","monitor":"primary","state":"visible","width":800,"height":600}"#
        );
        let parsed: SurfaceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }
}
