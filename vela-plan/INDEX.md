# INDEX — плоский индекс задач

Генерируется из `manifest.json`. Всего: **8 этапов**,
**39 задач**, **120 подзадач**,
**57.0 человеко-дней** оценки.

Правило: ссылка ведёт на ТЗ. Идентификаторы неизменяемы.

### P1 — Каркас и контракты · gate G1 · 4.0 дн.

План этапа: [`01-foundations/README.md`](01-foundations/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0101`](01-foundations/T-0101-repo-toolchain/README.md) | T-0101 — Репозиторий, workspace, toolchain, CI | P0 | 1.0 | 4 | — |
| [`T-0102`](01-foundations/T-0102-ipc-contracts/README.md) | T-0102 — Контракты IPC и генерация TS-типов | P0 | 1.0 | 3 | T-0101 |
| [`T-0103`](01-foundations/T-0103-design-tokens/README.md) | T-0103 — Дизайн-токены, темы, ThemeService | P0 | 1.0 | 3 | T-0101 |
| [`T-0104`](01-foundations/T-0104-sqlite/README.md) | T-0104 — SQLite-схема, миграции, репозитории | P0 | 1.0 | 3 | T-0101 |

**T-0101** — `01-foundations/T-0101-repo-toolchain`
- [`ST-0101-01`](01-foundations/T-0101-repo-toolchain/ST-0101-01-cargo-workspace.md) — cargo-workspace
- [`ST-0101-02`](01-foundations/T-0101-repo-toolchain/ST-0101-02-vite-ts.md) — vite-ts
- [`ST-0101-03`](01-foundations/T-0101-repo-toolchain/ST-0101-03-lints.md) — lints
- [`ST-0101-04`](01-foundations/T-0101-repo-toolchain/ST-0101-04-ci.md) — ci

**T-0102** — `01-foundations/T-0102-ipc-contracts`
- [`ST-0102-01`](01-foundations/T-0102-ipc-contracts/ST-0102-01-ipc-types.md) — ipc-types
- [`ST-0102-02`](01-foundations/T-0102-ipc-contracts/ST-0102-02-codegen.md) — codegen
- [`ST-0102-03`](01-foundations/T-0102-ipc-contracts/ST-0102-03-invoke-wrapper.md) — invoke-wrapper

**T-0103** — `01-foundations/T-0103-design-tokens`
- [`ST-0103-01`](01-foundations/T-0103-design-tokens/ST-0103-01-tokens.md) — tokens
- [`ST-0103-02`](01-foundations/T-0103-design-tokens/ST-0103-02-theme-service.md) — theme-service
- [`ST-0103-03`](01-foundations/T-0103-design-tokens/ST-0103-03-glass-primitive.md) — glass-primitive

**T-0104** — `01-foundations/T-0104-sqlite`
- [`ST-0104-01`](01-foundations/T-0104-sqlite/ST-0104-01-db-layer.md) — db-layer
- [`ST-0104-02`](01-foundations/T-0104-sqlite/ST-0104-02-migrations.md) — migrations
- [`ST-0104-03`](01-foundations/T-0104-sqlite/ST-0104-03-repositories.md) — repositories


### P2 — Оболочка: окна и поверхности · gate G2 · 8.0 дн.

План этапа: [`02-shell/README.md`](02-shell/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0201`](02-shell/T-0201-tauri-app/README.md) | T-0201 — Приложение Tauri | P0 | 1.5 | 3 | T-0101, T-0102, T-0104 |
| [`T-0202`](02-shell/T-0202-workerw-scene/README.md) | T-0202 — Поверхность A (сцена) | P0 | 2.0 | 3 | T-0201 |
| [`T-0203`](02-shell/T-0203-ui-surface/README.md) | T-0203 — Поверхность B (UI) | P0 | 2.0 | 3 | T-0201 |
| [`T-0204`](02-shell/T-0204-autostart-power/README.md) | T-0204 — Автозапуск и питание | P0 | 1.5 | 3 | T-0202, T-0203 |
| [`T-0205`](02-shell/T-0205-overlay-hotkeys/README.md) | T-0205 — Поверхность C и хоткеи | P1 | 1.0 | 2 | T-0203 |

**T-0201** — `02-shell/T-0201-tauri-app`
- [`ST-0201-01`](02-shell/T-0201-tauri-app/ST-0201-01-surface-manager.md) — surface-manager
- [`ST-0201-02`](02-shell/T-0201-tauri-app/ST-0201-02-tray.md) — tray
- [`ST-0201-03`](02-shell/T-0201-tauri-app/ST-0201-03-single-instance.md) — single-instance

**T-0202** — `02-shell/T-0202-workerw-scene`
- [`ST-0202-01`](02-shell/T-0202-workerw-scene/ST-0202-01-workerw-parent.md) — workerw-parent
- [`ST-0202-02`](02-shell/T-0202-workerw-scene/ST-0202-02-geometry-dpi.md) — geometry-dpi
- [`ST-0202-03`](02-shell/T-0202-workerw-scene/ST-0202-03-explorer-recovery.md) — explorer-recovery

**T-0203** — `02-shell/T-0203-ui-surface`
- [`ST-0203-01`](02-shell/T-0203-ui-surface/ST-0203-01-create.md) — create
- [`ST-0203-02`](02-shell/T-0203-ui-surface/ST-0203-02-hit-test.md) — hit-test
- [`ST-0203-03`](02-shell/T-0203-ui-surface/ST-0203-03-modes-focus.md) — modes-focus

**T-0204** — `02-shell/T-0204-autostart-power`
- [`ST-0204-01`](02-shell/T-0204-autostart-power/ST-0204-01-autostart.md) — autostart
- [`ST-0204-02`](02-shell/T-0204-autostart-power/ST-0204-02-power-policy.md) — power-policy
- [`ST-0204-03`](02-shell/T-0204-autostart-power/ST-0204-03-session-events.md) — session-events

**T-0205** — `02-shell/T-0205-overlay-hotkeys`
- [`ST-0205-01`](02-shell/T-0205-overlay-hotkeys/ST-0205-01-overlay-lifecycle.md) — overlay-lifecycle
- [`ST-0205-02`](02-shell/T-0205-overlay-hotkeys/ST-0205-02-hotkeys.md) — hotkeys


### P3 — Бесконечный холст · gate G3 · 9.0 дн.

План этапа: [`03-canvas/README.md`](03-canvas/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0301`](03-canvas/T-0301-world-viewport/README.md) | T-0301 — Мир и вьюпорт | P0 | 2.0 | 4 | T-0103, T-0203 |
| [`T-0302`](03-canvas/T-0302-layout-model/README.md) | T-0302 — Модель раскладки | P0 | 2.0 | 4 | T-0301, T-0104 |
| [`T-0303`](03-canvas/T-0303-interaction/README.md) | T-0303 — Взаимодействие | P0 | 2.5 | 4 | T-0302 |
| [`T-0304`](03-canvas/T-0304-glass/README.md) | T-0304 — Стекло | P0 | 1.5 | 3 | T-0301 |
| [`T-0305`](03-canvas/T-0305-profiles/README.md) | T-0305 — Профили | P1 | 1.0 | 2 | T-0302 |

**T-0301** — `03-canvas/T-0301-world-viewport`
- [`ST-0301-01`](03-canvas/T-0301-world-viewport/ST-0301-01-camera.md) — camera
- [`ST-0301-02`](03-canvas/T-0301-world-viewport/ST-0301-02-grid-fit.md) — grid-fit
- [`ST-0301-03`](03-canvas/T-0301-world-viewport/ST-0301-03-virtualization.md) — virtualization
- [`ST-0301-04`](03-canvas/T-0301-world-viewport/ST-0301-04-render-loop.md) — render-loop

**T-0302** — `03-canvas/T-0302-layout-model`
- [`ST-0302-01`](03-canvas/T-0302-layout-model/ST-0302-01-element-types.md) — element-types
- [`ST-0302-02`](03-canvas/T-0302-layout-model/ST-0302-02-apply-patch.md) — apply-patch
- [`ST-0302-03`](03-canvas/T-0302-layout-model/ST-0302-03-zustand-store.md) — zustand-store
- [`ST-0302-04`](03-canvas/T-0302-layout-model/ST-0302-04-orphaned.md) — orphaned

**T-0303** — `03-canvas/T-0303-interaction`
- [`ST-0303-01`](03-canvas/T-0303-interaction/ST-0303-01-drag.md) — drag
- [`ST-0303-02`](03-canvas/T-0303-interaction/ST-0303-02-snap-guides.md) — snap-guides
- [`ST-0303-03`](03-canvas/T-0303-interaction/ST-0303-03-selection.md) — selection
- [`ST-0303-04`](03-canvas/T-0303-interaction/ST-0303-04-zorder-keys.md) — zorder-keys

**T-0304** — `03-canvas/T-0304-glass`
- [`ST-0304-01`](03-canvas/T-0304-glass/ST-0304-01-mirror.md) — mirror
- [`ST-0304-02`](03-canvas/T-0304-glass/ST-0304-02-glass-render.md) — glass-render
- [`ST-0304-03`](03-canvas/T-0304-glass/ST-0304-03-degradation-budget.md) — degradation-budget

**T-0305** — `03-canvas/T-0305-profiles`
- [`ST-0305-01`](03-canvas/T-0305-profiles/ST-0305-01-profile-model.md) — profile-model
- [`ST-0305-02`](03-canvas/T-0305-profiles/ST-0305-02-activate.md) — activate


### P4 — Рантайм виджетов и базовые модули · gate G4 · 8.0 дн.

План этапа: [`04-widget-core/README.md`](04-widget-core/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0401`](04-widget-core/T-0401-runtime/README.md) | T-0401 — Рантайм виджетов | P0 | 2.5 | 4 | T-0302, T-0304 |
| [`T-0402`](04-widget-core/T-0402-host-card/README.md) | T-0402 — Хост-карточка | P0 | 1.5 | 3 | T-0401, T-0304 |
| [`T-0403`](04-widget-core/T-0403-clock/README.md) | T-0403 — Часы | P0 | 1.0 | 3 | T-0402 |
| [`T-0404`](04-widget-core/T-0404-notes/README.md) | T-0404 — Заметки | P0 | 1.5 | 3 | T-0402, T-0104 |
| [`T-0405`](04-widget-core/T-0405-timer/README.md) | T-0405 — Таймер | P0 | 1.5 | 3 | T-0402 |

**T-0401** — `04-widget-core/T-0401-runtime`
- [`ST-0401-01`](04-widget-core/T-0401-runtime/ST-0401-01-registry.md) — registry
- [`ST-0401-02`](04-widget-core/T-0401-runtime/ST-0401-02-context.md) — context
- [`ST-0401-03`](04-widget-core/T-0401-runtime/ST-0401-03-boundary.md) — boundary
- [`ST-0401-04`](04-widget-core/T-0401-runtime/ST-0401-04-visibility.md) — visibility

**T-0402** — `04-widget-core/T-0402-host-card`
- [`ST-0402-01`](04-widget-core/T-0402-host-card/ST-0402-01-card.md) — card
- [`ST-0402-02`](04-widget-core/T-0402-host-card/ST-0402-02-settings-form.md) — settings-form
- [`ST-0402-03`](04-widget-core/T-0402-host-card/ST-0402-03-add-placing.md) — add-placing

**T-0403** — `04-widget-core/T-0403-clock`
- [`ST-0403-01`](04-widget-core/T-0403-clock/ST-0403-01-digital.md) — digital
- [`ST-0403-02`](04-widget-core/T-0403-clock/ST-0403-02-timezones.md) — timezones
- [`ST-0403-03`](04-widget-core/T-0403-clock/ST-0403-03-analog.md) — analog

**T-0404** — `04-widget-core/T-0404-notes`
- [`ST-0404-01`](04-widget-core/T-0404-notes/ST-0404-01-note-model.md) — note-model
- [`ST-0404-02`](04-widget-core/T-0404-notes/ST-0404-02-checklist.md) — checklist
- [`ST-0404-03`](04-widget-core/T-0404-notes/ST-0404-03-text-md.md) — text-md

**T-0405** — `04-widget-core/T-0405-timer`
- [`ST-0405-01`](04-widget-core/T-0405-timer/ST-0405-01-machine.md) — machine
- [`ST-0405-02`](04-widget-core/T-0405-timer/ST-0405-02-ui-ring.md) — ui-ring
- [`ST-0405-03`](04-widget-core/T-0405-timer/ST-0405-03-notify-sound.md) — notify-sound


### P5 — Захват экрана, аннотации, pin · gate G5 · 6.0 дн.

План этапа: [`05-capture/README.md`](05-capture/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0501`](05-capture/T-0501-wgc-capture/README.md) | T-0501 — Захват через WGC | P0 | 2.0 | 3 | T-0205, T-0202 |
| [`T-0502`](05-capture/T-0502-region-overlay/README.md) | T-0502 — Оверлей выделения | P0 | 1.5 | 3 | T-0501, T-0205 |
| [`T-0503`](05-capture/T-0503-annotator/README.md) | T-0503 — Аннотатор | P0 | 1.5 | 3 | T-0502 |
| [`T-0504`](05-capture/T-0504-pin-ocr/README.md) | T-0504 — Pin и OCR | P1 | 1.0 | 3 | T-0503, T-0302 |

**T-0501** — `05-capture/T-0501-wgc-capture`
- [`ST-0501-01`](05-capture/T-0501-wgc-capture/ST-0501-01-d3d-session.md) — d3d-session
- [`ST-0501-02`](05-capture/T-0501-wgc-capture/ST-0501-02-encode-hdr.md) — encode-hdr
- [`ST-0501-03`](05-capture/T-0501-wgc-capture/ST-0501-03-capture-ipc.md) — capture-ipc

**T-0502** — `05-capture/T-0502-region-overlay`
- [`ST-0502-01`](05-capture/T-0502-region-overlay/ST-0502-01-overlay-photo.md) — overlay-photo
- [`ST-0502-02`](05-capture/T-0502-region-overlay/ST-0502-02-draw-snap-loupe.md) — draw-snap-loupe
- [`ST-0502-03`](05-capture/T-0502-region-overlay/ST-0502-03-confirm-flow.md) — confirm-flow

**T-0503** — `05-capture/T-0503-annotator`
- [`ST-0503-01`](05-capture/T-0503-annotator/ST-0503-01-canvas-toolbar.md) — canvas-toolbar
- [`ST-0503-02`](05-capture/T-0503-annotator/ST-0503-02-shapes.md) — shapes
- [`ST-0503-03`](05-capture/T-0503-annotator/ST-0503-03-privacy-export.md) — privacy-export

**T-0504** — `05-capture/T-0504-pin-ocr`
- [`ST-0504-01`](05-capture/T-0504-pin-ocr/ST-0504-01-pin-element.md) — pin-element
- [`ST-0504-02`](05-capture/T-0504-pin-ocr/ST-0504-02-ocr.md) — ocr
- [`ST-0504-03`](05-capture/T-0504-pin-ocr/ST-0504-03-history.md) — history


### P6 — Живой фон · gate G6 · 9.0 дн.

План этапа: [`06-scene/README.md`](06-scene/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0601`](06-scene/T-0601-scene-engine/README.md) | T-0601 — Движок сцены | P0 | 2.5 | 4 | T-0304, T-0202 |
| [`T-0602`](06-scene/T-0602-photo-video/README.md) | T-0602 — Фото и видео | P0 | 2.0 | 3 | T-0601 |
| [`T-0603`](06-scene/T-0603-2d-3d/README.md) | T-0603 — 2D/3D сцена | P0 | 2.5 | 3 | T-0601 |
| [`T-0604`](06-scene/T-0604-audio-game/README.md) | T-0604 — Аудио и игра | P1 | 1.0 | 3 | T-0603, T-0602 |
| [`T-0605`](06-scene/T-0605-power-scene/README.md) | T-0605 — Энергополитика | P0 | 1.0 | 3 | T-0602, T-0204 |

**T-0601** — `06-scene/T-0601-scene-engine`
- [`ST-0601-01`](06-scene/T-0601-scene-engine/ST-0601-01-scene-model.md) — scene-model
- [`ST-0601-02`](06-scene/T-0601-scene-engine/ST-0601-02-webgl-renderer.md) — webgl-renderer
- [`ST-0601-03`](06-scene/T-0601-scene-engine/ST-0601-03-quality.md) — quality
- [`ST-0601-04`](06-scene/T-0601-scene-engine/ST-0601-04-transitions.md) — transitions

**T-0602** — `06-scene/T-0602-photo-video`
- [`ST-0602-01`](06-scene/T-0602-photo-video/ST-0602-01-photo.md) — photo
- [`ST-0602-02`](06-scene/T-0602-photo-video/ST-0602-02-slideshow.md) — slideshow
- [`ST-0602-03`](06-scene/T-0602-photo-video/ST-0602-03-video.md) — video

**T-0603** — `06-scene/T-0603-2d-3d`
- [`ST-0603-01`](06-scene/T-0603-2d-3d/ST-0603-01-shader-pipeline.md) — shader-pipeline
- [`ST-0603-02`](06-scene/T-0603-2d-3d/ST-0603-02-shaders-fbm.md) — shaders-fbm
- [`ST-0603-03`](06-scene/T-0603-2d-3d/ST-0603-03-flow-particles.md) — flow-particles

**T-0604** — `06-scene/T-0604-audio-game`
- [`ST-0604-01`](06-scene/T-0604-audio-game/ST-0604-01-wasapi-fft.md) — wasapi-fft
- [`ST-0604-02`](06-scene/T-0604-audio-game/ST-0604-02-reactive-render.md) — reactive-render
- [`ST-0604-03`](06-scene/T-0604-audio-game/ST-0604-03-game-skeleton.md) — game-skeleton

**T-0605** — `06-scene/T-0605-power-scene`
- [`ST-0605-01`](06-scene/T-0605-power-scene/ST-0605-01-policy.md) — policy
- [`ST-0605-02`](06-scene/T-0605-power-scene/ST-0605-02-battery-measure.md) — battery-measure
- [`ST-0605-03`](06-scene/T-0605-power-scene/ST-0605-03-indicator.md) — indicator


### P7 — Экосистема модулей · gate G7 · 7.0 дн.

План этапа: [`07-modules/README.md`](07-modules/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0701`](07-modules/T-0701-calendar/README.md) | T-0701 — Календарь | P0 | 1.5 | 3 | T-0402, T-0104 |
| [`T-0702`](07-modules/T-0702-sysmon/README.md) | T-0702 — Системный монитор | P0 | 1.0 | 3 | T-0402 |
| [`T-0703`](07-modules/T-0703-weather/README.md) | T-0703 — Погода | P1 | 1.0 | 3 | T-0402 |
| [`T-0704`](07-modules/T-0704-cron/README.md) | T-0704 — Планировщик | P0 | 1.5 | 3 | T-0402, T-0204 |
| [`T-0705`](07-modules/T-0705-player/README.md) | T-0705 — Плеер | P1 | 1.0 | 3 | T-0402, T-0604 |
| [`T-0706`](07-modules/T-0706-command-palette/README.md) | T-0706 — Палитра и буфер | P0 | 1.0 | 3 | T-0402, T-0205 |

**T-0701** — `07-modules/T-0701-calendar`
- [`ST-0701-01`](07-modules/T-0701-calendar/ST-0701-01-ics-parser.md) — ics-parser
- [`ST-0701-02`](07-modules/T-0701-calendar/ST-0701-02-sync.md) — sync
- [`ST-0701-03`](07-modules/T-0701-calendar/ST-0701-03-widget.md) — widget

**T-0702** — `07-modules/T-0702-sysmon`
- [`ST-0702-01`](07-modules/T-0702-sysmon/ST-0702-01-metrics.md) — metrics
- [`ST-0702-02`](07-modules/T-0702-sysmon/ST-0702-02-widget.md) — widget
- [`ST-0702-03`](07-modules/T-0702-sysmon/ST-0702-03-top-processes.md) — top-processes

**T-0703** — `07-modules/T-0703-weather`
- [`ST-0703-01`](07-modules/T-0703-weather/ST-0703-01-api-client.md) — api-client
- [`ST-0703-02`](07-modules/T-0703-weather/ST-0703-02-widget.md) — widget
- [`ST-0703-03`](07-modules/T-0703-weather/ST-0703-03-city-settings.md) — city-settings

**T-0704** — `07-modules/T-0704-cron`
- [`ST-0704-01`](07-modules/T-0704-cron/ST-0704-01-cron-parser.md) — cron-parser
- [`ST-0704-02`](07-modules/T-0704-cron/ST-0704-02-scheduler-thread.md) — scheduler-thread
- [`ST-0704-03`](07-modules/T-0704-cron/ST-0704-03-widget-editor.md) — widget-editor

**T-0705** — `07-modules/T-0705-player`
- [`ST-0705-01`](07-modules/T-0705-player/ST-0705-01-gsmtc.md) — gsmtc
- [`ST-0705-02`](07-modules/T-0705-player/ST-0705-02-widget.md) — widget
- [`ST-0705-03`](07-modules/T-0705-player/ST-0705-03-viz-empty.md) — viz-empty

**T-0706** — `07-modules/T-0706-command-palette`
- [`ST-0706-01`](07-modules/T-0706-command-palette/ST-0706-01-palette-core.md) — palette-core
- [`ST-0706-02`](07-modules/T-0706-command-palette/ST-0706-02-clipboard.md) — clipboard
- [`ST-0706-03`](07-modules/T-0706-command-palette/ST-0706-03-ticker.md) — ticker


### P8 — Упрочнение и релиз · gate G8 · 6.0 дн.

План этапа: [`08-release/README.md`](08-release/README.md)

| Задача | Название | Приоритет | Дней | Подзадач | Зависит от |
|---|---|---|---|---|---|
| [`T-0801`](08-release/T-0801-environment-matrix/README.md) | T-0801 — Матрица окружений | P0 | 1.5 | 3 | T-0302, T-0203 |
| [`T-0802`](08-release/T-0802-performance-soak/README.md) | T-0802 — Производительность и soak | P0 | 1.0 | 2 | T-0301, T-0401, T-0602 |
| [`T-0803`](08-release/T-0803-installer/README.md) | T-0803 — Установщик | P0 | 1.5 | 3 | — |
| [`T-0804`](08-release/T-0804-settings-onboarding/README.md) | T-0804 — Настройки и онбординг | P1 | 1.0 | 3 | T-0303, T-0402 |
| [`T-0805`](08-release/T-0805-observability/README.md) | T-0805 — Наблюдаемость | P1 | 1.0 | 3 | — |

**T-0801** — `08-release/T-0801-environment-matrix`
- [`ST-0801-01`](08-release/T-0801-environment-matrix/ST-0801-01-dpi-glass.md) — dpi-glass
- [`ST-0801-02`](08-release/T-0801-environment-matrix/ST-0801-02-monitor-events.md) — monitor-events
- [`ST-0801-03`](08-release/T-0801-environment-matrix/ST-0801-03-rdp-vm.md) — rdp-vm

**T-0802** — `08-release/T-0802-performance-soak`
- [`ST-0802-01`](08-release/T-0802-performance-soak/ST-0802-01-runs.md) — runs
- [`ST-0802-02`](08-release/T-0802-performance-soak/ST-0802-02-leaks.md) — leaks

**T-0803** — `08-release/T-0803-installer`
- [`ST-0803-01`](08-release/T-0803-installer/ST-0803-01-installer-config.md) — installer-config
- [`ST-0803-02`](08-release/T-0803-installer/ST-0803-02-updater.md) — updater
- [`ST-0803-03`](08-release/T-0803-installer/ST-0803-03-uninstall.md) — uninstall

**T-0804** — `08-release/T-0804-settings-onboarding`
- [`ST-0804-01`](08-release/T-0804-settings-onboarding/ST-0804-01-settings-window.md) — settings-window
- [`ST-0804-02`](08-release/T-0804-settings-onboarding/ST-0804-02-onboarding.md) — onboarding
- [`ST-0804-03`](08-release/T-0804-settings-onboarding/ST-0804-03-a11y.md) — a11y

**T-0805** — `08-release/T-0805-observability`
- [`ST-0805-01`](08-release/T-0805-observability/ST-0805-01-logging.md) — logging
- [`ST-0805-02`](08-release/T-0805-observability/ST-0805-02-diagnostics-report.md) — diagnostics-report
- [`ST-0805-03`](08-release/T-0805-observability/ST-0805-03-crash.md) — crash

