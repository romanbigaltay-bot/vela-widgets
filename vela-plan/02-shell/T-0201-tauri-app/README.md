---
id: T-0201
stage: P2
title: Приложение Tauri: трей, единственный экземпляр, окна-скелеты
status: done
est: 1.5d
owner: rust
depends: [T-0101, T-0102, T-0104]
gate: G2
---

# T-0201 — Приложение Tauri

## 0. Фактическое состояние (2026-11-15)

T-0201 реализована и закрыта 2026-11-15. Приложение — Tauri 2.12.5
(`tauri-build` 2.7.4, `tauri-plugin-single-instance` 2.4.x) на Windows; на
других ОС собирается headless-заглушка, чтобы workspace проходил Linux CI.

- `tauri.conf.json`: `app.windows: []`; единственная поверхность UI 800×600
  создаётся кодом в `apps/desktop/src/surfaces.rs`, сразу скрытой.
- `SurfaceManager` — реестр `HashMap<SurfaceId, SurfaceHandle>`; `ensure`
  идемпотентен по `(kind, monitor)`, `status()` отдаёт снимок, `Drop`
  уничтожает окно. Win32-вызовы — только через `vela-platform::win`.
- Стили UI: `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` (плюс `WS_EX_WINDOWEDGE`
  и `WS_EX_ACCEPTFILES` от tao); `WS_EX_APPWINDOW` снимается после создания
  и повторно перед показом, потому что tao переписывает `GWL_EXSTYLE`
  целиком. Фактический ex-style — `0x08000190`; в панели задач и Alt+Tab
  окна нет.
- Показ/скрытие — через `ShowWindow(SW_SHOWNOACTIVATE/SW_HIDE)`, не через
  `window.show()/hide()`; после показа вызывается `SetForegroundWindow`,
  чтобы реальный Alt+F4 доходил до поверхности. Alt+F4 скрывает окно,
  процесс остаётся жив.
- Трей: левый клик — показать/скрыть UI, меню — «Показать виджеты» и
  «Выход». Чекбокс читает состояние из `SurfaceManager`. Выход — только
  через трей; `ExitRequested` без кода отменяется.
- Single instance: второй процесс выходит с кодом 0 и просит первый
  показать UI. Тост «Vela уже запущена» отложен.
- CLI: `--settings`, `--no-scene`, `--safe-mode` разбираются и хранятся;
  потребители появятся в T-0202/T-0204.
- IPC: `window.surfaces_status` → `Envelope<Vec<SurfaceStatus>>`; событие
  `window:surfaces_changed` (точка в имени недопустима в Tauri 2).
  UI-панель `SurfacesPanel` вызывает команду и подписывается на событие.
- Замеры release (2026-11-15, прямой `CreateProcess`, 3 прогона): окно UI
  создаётся скрытым за 84–169 мс, трей появляется за 656–692 мс; второй
  экземпляр живёт 44.7–45.4 мс и выходит с кодом 0; «Выход» — код 0,
  процессов не остаётся.
- Проверки: 148 Rust-тестов (`--all-features`), 76 UI-тестов, `cargo fmt`,
  clippy, `pnpm typecheck/lint/format:check/ipc:check`, `arch-check` —
  зелёные; ручной смоук подтверждён (трей, показ/скрытие, панель, Alt+F4,
  второй запуск, выход).
- Отклонения: `transparent: false` (T-0203), `parent_to_workerw` возвращает
  `Unsupported` (T-0202), `SurfaceState` только `Hidden | Visible`, меню трея
  сокращено до двух пунктов, `Ctrl+Alt+Q` и тост второго экземпляра не
  реализованы, плагины `autostart`/`window-state`/`opener` не подключены
  (T-0204/T-0205).
- Режим UI: `cargo run -p vela` — dev-режим Tauri, окно грузит `devUrl`
  (`http://localhost:5173`), поэтому нужен `pnpm dev`; автономный бинарь
  собирается с `--features tauri/custom-protocol` после `pnpm build`
  (D-27). Проверено 2026-11-15: release-бинарь с этой фичей открывает
  `http://tauri.localhost/`, `document.title` — «Vela».

Статус — `done` (2026-11-15).

## 1. Цель

Работающее приложение с треем и менеджером окон, из которого T-0202 и T-0203
создают поверхности. Никакого контента — только каркас жизни приложения.

## 2. Решение

- `tauri.conf.json`: `app.windows` пуст; все окна создаются кодом (нужен полный
  контроль над `WS_EX_*` до показа).
- Плагины: в T-0201 подключён только `single-instance`. `autostart`
  (T-0204), `window-state` (окно настроек) и `opener` (ссылки) добавляются
  вместе с потребителями.
- Менеджер `SurfaceManager`: реестр поверхностей `{ kind, monitor, handle, state }`,
  единая точка создания/уничтожения и восстановления.
- Трей: иконка в двух вариантах (тёмная/светлая тема панели задач), меню из
  `TrayMenu` с актуальными чекбоксами.
- Выход только через трей (защита от случайного закрытия). `Ctrl+Alt+Q`
  появится в T-0205.

## 3. Контракты

```rust
pub enum SurfaceKind { Scene, Ui, Overlay }

pub struct SurfaceSpec {
    pub kind: SurfaceKind,
    pub monitor: MonitorId,
    pub transparent: bool,
    pub click_through: bool,
    pub no_activate: bool,
    pub top_most: bool,
    pub parent_to_workerw: bool,
    pub width: u32,
    pub height: u32,
}

pub struct SurfaceStatus {
    pub kind: SurfaceKind,
    pub monitor: MonitorId,
    pub state: SurfaceState,
    pub width: u32,
    pub height: u32,
}

impl SurfaceManager {
    pub(crate) fn ensure(
        &mut self,
        app: &AppHandle<Wry>,
        spec: SurfaceSpec,
    ) -> Result<SurfaceId, SurfaceError>;
    pub(crate) fn destroy(&mut self, id: SurfaceId) -> Result<(), SurfaceError>;
    pub(crate) fn destroy_all(&mut self);
    pub(crate) fn status(&self) -> Vec<SurfaceStatus>;
    pub(crate) fn has_kind(&self, kind: SurfaceKind) -> bool;
    pub(crate) fn kind_visible(&self, kind: SurfaceKind) -> bool;
    pub(crate) fn set_kind_visible(
        &mut self,
        kind: SurfaceKind,
        visible: bool,
    ) -> Result<(), SurfaceError>;
    pub(crate) fn toggle_kind(&mut self, kind: SurfaceKind) -> Result<bool, SurfaceError>;
}
```

IPC: `window.surfaces_status` → `Envelope<Vec<SurfaceStatus>>`; событие
`window:surfaces_changed`. В плане было `window.surfaces_changed`, но Tauri 2
разрешает в именах событий только буквы, цифры, `-`, `/`, `:` и `_`, поэтому
точка заменена двоеточием.

## 4. Состав подзадач

| ID | Подзадача | Дней |
|---|---|---|
| ST-0201-01 | Конфиг Tauri, менеджер поверхностей, создание окон | 0.5 |
| ST-0201-02 | Трей и меню | 0.5 |
| ST-0201-03 | Single instance, обработка argv, выход/скрытие | 0.5 |

## 5. Критерии приёмки

- **AC-1** Приложение стартует без окон за ≤ 800 мс и живёт только в трее.
- **AC-2** Поверхности создаются кодом с нужными флагами (проверяется
  `GetWindowLongPtr` в интеграционном тесте).
- **AC-3** Меню трея отражает реальное состояние (чекбоксы не врут).
- **AC-4** Второй экземпляр завершает работу с кодом 0, активируя первый.
- **AC-5** Alt+F4 на поверхности B скрывает её, приложение живо.
- **AC-6** Процесс не остаётся после «Выход» (проверка по `tasklist`).

### Подтверждение AC (2026-11-15)

| AC | Доказательство |
|---|---|
| AC-1 | Release, прямой `CreateProcess`, 3 прогона: окно UI создаётся скрытым за 84–169 мс, трей появляется за 656–692 мс — оба значения ниже 800 мс |
| AC-2 | `GetWindowLongPtrW(GWL_EXSTYLE)` = `0x08000190` (NOACTIVATE, TOOLWINDOW, WINDOWEDGE, ACCEPTFILES); `WS_EX_APPWINDOW` отсутствует; вручную: окна нет в панели задач и Alt+Tab; тесты `ex_style_round_trips`, `remove_ex_style_clears_only_requested_bits` |
| AC-3 | Чекбокс трея читает `SurfaceManager::kind_visible`; ручной смоук: после показа/скрытия и после Alt+F4 галочка совпадает с реальной видимостью |
| AC-4 | Второй экземпляр: 44.7–45.4 мс, exit 0; первый остаётся жив, окно видимо; ручной смоук подтвердил активацию первого |
| AC-5 | Реальный Alt+F4 через `SendKeys` при UI в фокусе: окно скрылось, процесс жив, ex-style не изменился |
| AC-6 | WM_COMMAND `1001` («Выход»): exit 0, `Get-Process vela` — 0 процессов |

## 6. Тест-план

| ID | Проверка |
|---|---|
| TC-0201-01 | Интеграция: создать/уничтожить поверхность 20 раз, хендлы не текут |
| TC-0201-02 | Ручная: второй запуск активирует первый |
| TC-0201-03 | Ручная: выход через трей, процесс завершён |

## 7. Риски

- Создание окон из `setup` до готовности WebView2 → создавать после
  `on_page_load`, иначе белый кадр.
- Утечка хендлов при частом пересоздании — тест на 20 циклов обязателен.
