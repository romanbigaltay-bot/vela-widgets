---
id: ST-0301-01
parent: T-0301
title: Camera, transform, зум к курсору, pan
status: ready
est: 0.7d
owner: ui
---

# ST-0301-01 — Камера

## Цель

Камера с предсказуемым поведением, тестируемая как чистая математика.

## Реализация

1. `shared/lib/camera.ts`: чистые функции `worldToScreen`, `screenToWorld`,
   `zoomAt(cam, pointer, nextScale)`, `clampScale`, `fitToContent(bounds, viewport)`.
2. `layoutStore.camera: Camera` + экшены `panBy`, `zoomAt`, `setCamera`,
   `fit`. Во время жеста — `setCamera` напрямую (без IPC).
3. Жест pan: `pointerdown` на фоне → `pointermove` → `pointerup`; `Space` включает
   режим «рука» (курсор `grab`), `Esc` отменяет.
4. Колесо: `preventDefault`, зум; `Shift+wheel` — горизонтальный pan,
   `Ctrl+wheel` — чувствительный зум (для тачпадов).
5. Запись в БД: `layout.set_camera` по `pointerup`/`wheel` с дебаунсом 250 мс.
6. `Fit`: bounding box всех элементов + отступ 80 px, ограничение `scale` [0.35, 1.15].

## Критерии приёмки

- AC-1 Инвариант зума: `screenToWorld(pointer)` до и после зума совпадает ±0.5 px.
- AC-2 `fitToContent` при одном элементе не увеличивает сверх 1.15.
- AC-3 Камера сохраняется и восстанавливается при перезапуске.

## Тест-план

TC-0301-01, TC-0301-02 + property-тест на round-trip координат.

## Риски

Смешение DPR и CSS-пикселей: камера живёт в CSS-пикселях, DPR учитывается
только при отрисовке canvas-слоёв (стекло, рисунки).
