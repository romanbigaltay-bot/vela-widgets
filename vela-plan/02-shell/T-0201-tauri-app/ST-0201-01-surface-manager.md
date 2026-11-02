---
id: ST-0201-01
parent: T-0201
title: Конфиг Tauri и менеджер поверхностей
status: ready
est: 0.5d
owner: rust
---

# ST-0201-01 — Менеджер поверхностей

## Цель

Единый код создания окон с нужными Win32-флагами и реестром состояний.

## Реализация

1. `tauri.conf.json`: `app.windows: []`, `transparent: true` в дефолтном
   `window`-конфиге не нужен (создаём кодом), `withGlobalTauri: false`.
2. `platform::surface`: `create(spec)`:
   - `WebviewWindowBuilder` → `.transparent(spec.transparent)` →
     `.decorations(false)` → `.skip_taskbar(true)` → `.visible(false)` →
     `.focused(false)` → `.build()`;
   - до показа: `SetWindowLongPtrW(GWL_EXSTYLE)` добавляет `WS_EX_NOACTIVATE`
     (B, A), `WS_EX_TOOLWINDOW`, `WS_EX_TRANSPARENT` (A, при pass-through B);
   - `SetWindowPos(HWND_TOPMOST | SWP_NOACTIVATE)` для C.
3. `SurfaceManager` — `HashMap<SurfaceId, SurfaceHandle>`; `ensure` идемпотентен
   по `(kind, monitor)`.
4. Все Win32-вызовы — через обёртки в `vela-platform::win` с проверкой `GetLastError`.
5. `Drop` для `SurfaceHandle` закрывает окно; тест проверяет, что после
   `destroy()` хендл невалиден.

## Критерии приёмки

- AC-1 `GetWindowLongPtrW` на B содержит `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`.
- AC-2 Окно не появляется в панели задач и в Alt+Tab.
- AC-3 `ensure` дважды с одним ключом не создаёт второе окно.
- AC-4 20 циклов create/destroy — число хендлов процесса стабильно.

## Тест-план

Интеграционный тест с реальным окном (маркер `#[cfg(windows)]`), проверка стилей
через `GetWindowLongPtrW`; счётчик `GetGuiResources` до/после цикла.

## Риски

`WS_EX_TRANSPARENT` на A делает окно невидимым для кликов, но ломает
`SetParent(WorkerW)` — порядок операций фиксируется: сначала parent, потом стили.
