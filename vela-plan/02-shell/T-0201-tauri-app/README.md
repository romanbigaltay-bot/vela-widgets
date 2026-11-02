---
id: T-0201
stage: P2
title: Приложение Tauri: трей, единственный экземпляр, окна-скелеты
status: ready
est: 1.5d
owner: rust
depends: [T-0101, T-0102, T-0104]
gate: G2
---

# T-0201 — Приложение Tauri

## 1. Цель

Работающее приложение с треем и менеджером окон, из которого T-0202 и T-0203
создают поверхности. Никакого контента — только каркас жизни приложения.

## 2. Решение

- `tauri.conf.json`: `app.windows` пуст; все окна создаются кодом (нужен полный
  контроль над `WS_EX_*` до показа).
- Плагины: `single-instance`, `autostart` (используется в T-0204), `window-state`
  (только для окна настроек), `opener` (ссылки).
- Менеджер `SurfaceManager`: реестр поверхностей `{ kind, monitor, handle, state }`,
  единая точка создания/уничтожения и восстановления.
- Трей: иконка в двух вариантах (тёмная/светлая тема панели задач), меню из
  `TrayMenu` с актуальными чекбоксами.
- Выход только через трей или `Ctrl+Alt+Q` (защита от случайного закрытия).

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
}

impl SurfaceManager {
    pub fn ensure(&mut self, spec: SurfaceSpec) -> Result<SurfaceId>;
    pub fn rebuild_all(&mut self) -> Result<()>;   // для explorer restart
    pub fn destroy(&mut self, id: SurfaceId) -> Result<()>;
    pub fn status(&self) -> Vec<SurfaceStatus>;
}
```

IPC: `window.surfaces_status` → `Vec<SurfaceStatus>`; событие
`window.surfaces_changed`.

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
