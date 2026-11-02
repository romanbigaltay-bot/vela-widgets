---
id: ST-0203-01
parent: T-0203
title: Создание B: прозрачность, no-activate, no-taskbar
status: ready
est: 0.5d
owner: rust
---

# ST-0203-01 — Создание поверхности B

## Цель

Полноэкранное прозрачное окно на каждом мониторе с правильными стилями.

## Реализация

1. `create_ui(monitor)`: `WebviewWindowBuilder` с `transparent(true)`,
   `decorations(false)`, `shadow(false)`, `skip_taskbar(true)`, `visible(false)`.
2. После `build()`: `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, убрать `WS_CAPTION`.
3. `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE, DONOTROUND)`,
   `DWMWA_SYSTEMBACKDROP_TYPE = NONE` (иначе Windows добавит свою подложку).
4. Позиция — точно в физических границах монитора, `SWP_NOACTIVATE`.
5. HTML-фон — `background: transparent` в `index.html` для B; проверка, что
   WebView2 не заливает белым (`--disable-features=...` не требуется, но
   проверить `webview2.transparent` на практике).

## Критерии приёмки

- AC-1 Окно видно в `GetWindowRect` ровно по границам монитора.
- AC-2 Через пустое окно видна сцена A (не чёрный и не белый фон).
- AC-3 Нет записи в панели задач, нет в Alt+Tab.
- AC-4 `DwmGetWindowAttribute` подтверждает отсутствие системного backdrop.

## Тест-план

TC-0203-01; скриншот окна с включённой сценой A (визуальная проверка прозрачности).

## Риски

Известный баг WebView2: прозрачность работает только при `transparent: true`
**до** создания окна; при пересоздании нужно повторять флаг (не «патчить» окно).
