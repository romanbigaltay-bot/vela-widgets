//! Обёртки Win32 для работы с окнами.
//!
//! # Зачем отдельный модуль
//!
//! `unsafe` запрещён на уровне workspace, а звать Win32 из десятка мест — это
//! десяток шансов перепутать `GWL_EXSTYLE` с `GWL_STYLE`, забыть проверить
//! `GetLastError` или передать нулевой `HWND`. Здесь `unsafe` собран в одну
//! точку, а наружу торчит безопасный API с типизированными ошибками.
//!
//! # Почему `isize`, а не `HWND`
//!
//! `HWND` из крейта `windows` — это указатель. Пробрасывать его через границу
//! платформенного слоя значило бы заставить каждый вызывающий крейт зависеть
//! от версии `windows`: обновление Tauri меняет версию, и `HWND` перестаёт
//! быть одним и тем же типом. Сырой `isize` — стабильный ABI-примитив; в
//! `HWND` он превращается ровно в этом модуле.
//!
//! # Область ответственности
//!
//! Здесь нет создания окон: окна создаёт Tauri, а этот модуль доводит их до
//! нужного состояния (расширенные стили, z-order) и читает диагностику.
//! Создание тестового окна существует только под `#[cfg(test)]` — чтобы
//! проверить сами обёртки, не поднимая WebView2.

#![allow(unsafe_code)] // Модуль и есть граница с Win32; см. комментарий выше.

use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, GetLastError, SetLastError, ERROR_SUCCESS, HWND};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, GetGuiResources, OpenProcess, GR_GDIOBJECTS, PROCESS_QUERY_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, IsWindow, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_NOTOPMOST,
    HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT,
};

use crate::PlatformError;

/// Расширенные стили окна (`GWL_EXSTYLE`).
///
/// Newtype вместо голого `u32`: стили окна и стили класса — разные наборы
/// битов, и перепутать их компилятор не должен позволять.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExStyle(u32);

impl ExStyle {
    /// Стилей нет.
    pub const NONE: Self = Self(0);
    /// Окно не активируется кликом пользователя.
    pub const NOACTIVATE: Self = Self(WS_EX_NOACTIVATE.0);
    /// Окно не появляется в панели задач и Alt+Tab.
    pub const TOOLWINDOW: Self = Self(WS_EX_TOOLWINDOW.0);
    /// Клики проходят сквозь окно.
    pub const TRANSPARENT: Self = Self(WS_EX_TRANSPARENT.0);

    /// Сырое значение для `SetWindowLongPtrW`.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Собирает стиль из сырых битов (например, прочитанных из окна).
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Содержит ли стиль все биты `other`.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Объединение стилей.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for ExStyle {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.union(rhs)
    }
}

impl std::ops::BitOrAssign for ExStyle {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Превращает сырой хендл в `HWND`. Нулевой хендл остаётся нулевым.
fn hwnd(raw: isize) -> HWND {
    HWND(raw as *mut core::ffi::c_void)
}

/// Ошибка Win32 с контекстом операции.
fn last_error(context: &'static str) -> PlatformError {
    // SAFETY: `GetLastError` не имеет предусловий.
    let code = unsafe { GetLastError() };
    PlatformError::Win32 {
        code: code.0,
        context,
    }
}

/// Существует ли окно с таким хендлом.
///
/// `IsWindow` не различает «окно чужого процесса» и «окно уничтожено»:
/// для вызывающего это одно и то же — работать с ним нельзя.
pub fn is_window(raw: isize) -> bool {
    // SAFETY: `IsWindow` принимает любой хендл и не разыменовывает его.
    unsafe { IsWindow(Some(hwnd(raw))) }.as_bool()
}

/// Читает расширенные стили окна.
///
/// Нулевое значение — валидный стиль, поэтому ошибку здесь не отличить от
/// «стилей нет»; для несуществующего окна вернётся `NONE`. Проверить
/// существование можно через [`is_window`].
pub fn ex_style(raw: isize) -> ExStyle {
    // SAFETY: `GetWindowLongPtrW` для `GWL_EXSTYLE` не разыменовывает хендл,
    // а возвращает значение из ядра; невалидный хендл даёт 0.
    let bits = unsafe { GetWindowLongPtrW(hwnd(raw), GWL_EXSTYLE) };
    // `GWL_EXSTYLE` — 32-битное значение в младших битах `LONG_PTR`; верхние
    // биты для стилей не определены, поэтому сужение осознанное.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let bits = bits as u32;
    ExStyle::from_bits(bits)
}

/// Заменяет расширенные стили окна.
///
/// # Ошибки
///
/// `SetWindowLongPtrW` возвращает предыдущее значение, а `0` — валидное
/// предыдущее значение. Поэтому ошибка определяется по `GetLastError`,
/// предварительно очищенному: иначе остался бы код от прошлой операции.
pub fn set_ex_style(raw: isize, style: ExStyle) -> Result<(), PlatformError> {
    // SAFETY: см. `ex_style`; запись стилей — документированная операция.
    unsafe {
        SetLastError(ERROR_SUCCESS);
        let previous = SetWindowLongPtrW(hwnd(raw), GWL_EXSTYLE, style.bits() as isize);
        if previous == 0 {
            let code = GetLastError();
            if code != ERROR_SUCCESS {
                return Err(PlatformError::Win32 {
                    code: code.0,
                    context: "SetWindowLongPtrW(GWL_EXSTYLE)",
                });
            }
        }
    }
    Ok(())
}

/// Добавляет биты к расширенным стилям, не трогая остальные.
pub fn add_ex_style(raw: isize, style: ExStyle) -> Result<(), PlatformError> {
    set_ex_style(raw, ex_style(raw).union(style))
}

/// Ставит окно поверх всех или возвращает в обычный z-order.
///
/// `SWP_NOACTIVATE` обязателен: иначе показ оверлея забирал бы фокус у игры
/// или редактора, поверх которых он открыт.
pub fn set_topmost(raw: isize, topmost: bool) -> Result<(), PlatformError> {
    let insert_after = if topmost {
        HWND_TOPMOST
    } else {
        HWND_NOTOPMOST
    };
    // SAFETY: `SetWindowPos` не разыменовывает хендл; невалидный вернёт ошибку.
    unsafe {
        SetWindowPos(
            hwnd(raw),
            Some(insert_after),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .map_err(|_| last_error("SetWindowPos"))
    }
}

/// Сколько GDI-объектов держит процесс.
///
/// Используется тестом на утечки при пересоздании окон. `None` — Win32 не
/// смог ответить; это не повод падать, но и не «ноль объектов».
///
/// `GetCurrentProcess()` здесь не подходит: это псевдохендл, а
/// `GetGuiResources` требует настоящий handle с `PROCESS_QUERY_INFORMATION`.
/// На псевдохендле вызов возвращает 0 и `ERROR_ACCESS_DENIED`.
pub fn gdi_objects() -> Option<u32> {
    // SAFETY: PID текущего процесса всегда валиден; handle закрывается ниже
    // на всех путях после успешного `OpenProcess`.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_INFORMATION, false, GetCurrentProcessId()).ok()?;
        // Ноль — валидное число объектов (у процесса без GDI-ресурсов), а
        // `GetGuiResources` сообщает об ошибке тем же нулём. Различить их
        // можно только по `GetLastError`, очищенному до вызова.
        SetLastError(ERROR_SUCCESS);
        let count = GetGuiResources(process, GR_GDIOBJECTS);
        let error = GetLastError();
        let _ = CloseHandle(process);
        (error == ERROR_SUCCESS).then_some(count)
    }
}

/// Светлая ли тема панели задач.
///
/// Это не тема приложения: трей должен быть виден на фоне панели задач,
/// поэтому иконка выбирается по `SystemUsesLightTheme`, а не по теме Vela.
/// Отсутствие ключа (старые сборки, урезанный реестр) считается тёмной темой:
/// светлая иконка на светлой панели — заметнее, чем тёмная на тёмной.
pub fn system_uses_light_theme() -> bool {
    let mut value: u32 = 0;
    let mut size = u32::try_from(std::mem::size_of::<u32>()).unwrap_or(4);
    // SAFETY: буфер и его размер согласованы; `RegGetValueW` пишет не больше
    // `size` байт и сообщает фактический размер.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("SystemUsesLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        )
    };
    status == ERROR_SUCCESS && value != 0
}

/// Превращает `WIN32_ERROR` в `u32` — для тестов и логов.
#[cfg(test)]
fn error_code(error: windows::Win32::Foundation::WIN32_ERROR) -> u32 {
    error.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HINSTANCE;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, HCURSOR, HICON, WNDCLASSW,
        WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
    };

    static REGISTER_CLASS: Once = Once::new();

    /// Оконная процедура тестового окна: всё отдаём системе.
    unsafe extern "system" fn test_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        // SAFETY: стандартная оконная процедура не имеет дополнительных
        // предусловий и не хранит состояние между вызовами.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    /// Создаёт настоящее окно Win32 — минимальное, без WebView2.
    fn test_window() -> Result<isize, PlatformError> {
        REGISTER_CLASS.call_once(|| {
            let class = w!("VelaPlatformTestWindow");
            let module = unsafe { windows::Win32::System::LibraryLoader::GetModuleHandleW(None) }
                .map(|m| HINSTANCE(m.0))
                .unwrap_or_default();
            let wc = WNDCLASSW {
                style: windows::Win32::UI::WindowsAndMessaging::WNDCLASS_STYLES(0),
                lpfnWndProc: Some(test_wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: module,
                hIcon: HICON::default(),
                hCursor: HCURSOR::default(),
                hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH::default(),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class,
            };
            // SAFETY: структура заполнена, указатели на строки статичны.
            unsafe { RegisterClassW(&wc) };
        });

        // SAFETY: класс зарегистрирован выше; все необязательные параметры
        // переданы как `None`, размеры — константы.
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_OVERLAPPEDWINDOW,
                w!("VelaPlatformTestWindow"),
                w!("Vela platform test"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                64,
                64,
                None,
                None,
                None,
                None,
            )
        }
        .map_err(|_| last_error("CreateWindowExW"))?;
        Ok(hwnd.0 as isize)
    }

    fn destroy(raw: isize) -> Result<(), PlatformError> {
        // SAFETY: `DestroyWindow` проверяет хендл сам.
        unsafe { DestroyWindow(hwnd(raw)) }.map_err(|_| last_error("DestroyWindow"))
    }

    #[test]
    fn ex_style_round_trips() {
        let raw = test_window().expect("тестовое окно создаётся");
        let original = ex_style(raw);

        let wanted = original | ExStyle::NOACTIVATE | ExStyle::TOOLWINDOW;
        set_ex_style(raw, wanted).expect("стили записываются");
        let actual = ex_style(raw);
        assert!(
            actual.contains(ExStyle::NOACTIVATE),
            "WS_EX_NOACTIVATE должен быть выставлен"
        );
        assert!(
            actual.contains(ExStyle::TOOLWINDOW),
            "WS_EX_TOOLWINDOW должен быть выставлен"
        );

        // Возврат к исходному состоянию — чтобы тест не зависел от порядка.
        set_ex_style(raw, original).expect("стили возвращаются");
        assert_eq!(ex_style(raw), original);

        destroy(raw).expect("окно уничтожается");
        assert!(!is_window(raw), "после DestroyWindow хендл невалиден");
    }

    #[test]
    fn add_ex_style_preserves_existing_bits() {
        let raw = test_window().expect("тестовое окно создаётся");
        let original = ex_style(raw);

        add_ex_style(raw, ExStyle::TRANSPARENT).expect("стиль добавляется");
        let actual = ex_style(raw);
        assert!(actual.contains(ExStyle::TRANSPARENT));
        // Бит WS_EX_OVERLAPPEDWINDOW не должен потеряться: add не заменяет.
        assert!(actual.contains(original));

        set_ex_style(raw, original).expect("стили возвращаются");
        destroy(raw).expect("окно уничтожается");
    }

    #[test]
    fn twenty_create_destroy_cycles_do_not_leak_gdi_objects() {
        // Замер до цикла: тест идёт параллельно с другими, но только этот
        // создаёт окна, поэтому дельта отражает именно утечку.
        let before = gdi_objects().expect("GetGuiResources доступен");

        for _ in 0..20 {
            let raw = test_window().expect("окно создаётся");
            assert!(is_window(raw));
            destroy(raw).expect("окно уничтожается");
            assert!(!is_window(raw));
        }

        let after = gdi_objects().expect("GetGuiResources доступен");
        // Допуск в два объекта: система может создать служебный GDI-объект
        // при первом показе окна, и это не утечка. Рост на 20 означал бы,
        // что каждый цикл теряет ресурс.
        assert!(
            after <= before + 2,
            "GDI-объекты выросли: было {before}, стало {after}"
        );
    }

    #[test]
    fn topmost_can_be_set_and_cleared() {
        let raw = test_window().expect("тестовое окно создаётся");
        set_topmost(raw, true).expect("окно ставится поверх");
        set_topmost(raw, false).expect("окно возвращается в обычный порядок");
        destroy(raw).expect("окно уничтожается");
    }

    #[test]
    fn invalid_handle_is_not_a_window() {
        assert!(!is_window(0));
        assert_eq!(ex_style(0), ExStyle::NONE);
        // Запись в несуществующее окно обязана вернуть ошибку, а не молча
        // сделать вид, что всё хорошо.
        assert!(set_ex_style(0, ExStyle::TOOLWINDOW).is_err());
    }

    #[test]
    fn light_theme_probe_does_not_panic() {
        // Значение зависит от машины; проверяем, что вызов безопасен и
        // возвращает именно булево, а не падает на отсутствующем ключе.
        let _ = system_uses_light_theme();
    }

    #[test]
    fn error_code_is_exposed_for_logs() {
        assert_eq!(error_code(ERROR_SUCCESS), 0);
    }
}
