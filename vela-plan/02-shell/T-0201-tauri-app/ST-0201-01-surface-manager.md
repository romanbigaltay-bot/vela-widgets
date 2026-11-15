---
id: ST-0201-01
parent: T-0201
title: Конфиг Tauri и менеджер поверхностей
status: done
est: 0.5d
owner: rust
---

# ST-0201-01 — Менеджер поверхностей

## 0. Фактическое состояние (2026-11-15)

Реализовано в `apps/desktop/src/surfaces.rs` и `crates/vela-platform/src/win.rs`:

- `tauri.conf.json`: `app.windows: []`, окна создаются только кодом;
  `withGlobalTauri: false`, `csp: null` — до появления реального контента.
- `create_window`: `WebviewWindowBuilder` с `.decorations(false)`,
  `.skip_taskbar(true)`, `.visible(false)`, `.focused(false)`, `.center()`;
  размер 800×600, `transparent: false` до T-0203.
- `platform_style`: `WS_EX_TOOLWINDOW` всем поверхностям, `WS_EX_NOACTIVATE`
  при `no_activate`, `WS_EX_TRANSPARENT` при `click_through`; `top_most` —
  через `SetWindowPos(HWND_TOPMOST, SWP_NOACTIVATE)`.
- `WS_EX_APPWINDOW` снимается после создания и повторно перед каждым показом:
  tao переписывает `GWL_EXSTYLE` целиком и возвращает этот бит. Фактический
  ex-style UI — `0x08000190`.
- `ensure` идемпотентен по `(kind, monitor)`, `destroy` и `Drop` уничтожают
  окно ровно один раз, `status()` читает видимость из Win32.
- `parent_to_workerw: true` возвращает `SurfaceError::Unsupported` до T-0202.
- Показ/скрытие — `ShowWindow(SW_SHOWNOACTIVATE/SW_HIDE)`; после показа
  `SetForegroundWindow` (best-effort) и повторное применение стилей.

Подтверждение AC:

| AC | Доказательство |
|---|---|
| AC-1 | ex-style `0x08000190`; `WS_EX_NOACTIVATE` и `WS_EX_TOOLWINDOW` присутствуют, `WS_EX_APPWINDOW` отсутствует |
| AC-2 | вручную: окна нет в панели задач и Alt+Tab; стили проверены до и после показа |
| AC-3 | временный двойной вызов `ensure` в debug-сборке: один лог «поверхность создана» и одно окно `Tauri Window`; код откачен |
| AC-4 | `twenty_create_destroy_cycles_do_not_leak_gdi_objects` в `vela-platform` — 20 циклов, GDI-объекты не растут; оконные тесты сериализованы `WINDOW_TEST_LOCK` |

Отклонения: прозрачность и hit-regions — T-0203; WorkerW — T-0202;
`SurfaceState` пока только `Hidden | Visible`.

Статус — `done` (2026-11-15).

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
