//! Команды IPC.
//!
//! Первая команда P2 существует не для галочки: она доказывает, что цепочка
//! «React → `call()` → Tauri → Rust → `Envelope` → React» работает на живом
//! окне. До неё `InvokeMap` был пуст, и проверить round-trip было нечем.

use tauri::State;
use vela_ipc::{Envelope, SurfaceStatus};

use crate::state::{lock, AppState};

/// Возвращает снимок всех поверхностей.
///
/// Имя с точкой (`window.surfaces_status`) зафиксировано планом T-0201.
/// `rename` нужен потому, что имя Rust-функции не может содержать точку.
#[tauri::command(rename = "window.surfaces_status")]
pub(crate) fn window_surfaces_status(state: State<'_, AppState>) -> Envelope<Vec<SurfaceStatus>> {
    Envelope::ok(lock(&state.surfaces).status())
}
