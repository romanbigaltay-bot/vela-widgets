---
id: ST-0601-01
parent: T-0601
title: Модель сцены, SceneDigest, валидация
status: ready
est: 0.8d
owner: ui
---

# ST-0601-01 — Модель сцены

## Цель

Тип, который невозможно «случайно» сломать, и из которого выводится зеркало.

## Реализация

1. `scene/types.ts`: `SceneSource`, `SceneLayer`, `SceneConfig`, `SceneDigest`.
2. `validateScene(cfg)`: проверка `kind`, существования `assetId` (через IPC
   `assets.exists`), допустимых параметров шейдеров (схема).
3. `toDigest(cfg, size)`: `{ layers: [{type, colors|assetId, rect, opacity, blend}] }`
   — только то, что нужно зеркалу.
4. Сериализация в `profiles.scene_json` с `schemaVersion`; миграция старых
   профилей при чтении.
5. Дефолтные сцены: `aurora` (шейдер), `mesh`, `void`, «фото из папки».

## Критерии приёмки

- AC-1 Невалидная сцена возвращает структурированную ошибку с путём до поля.
- AC-2 `toDigest` не содержит ничего, кроме цветов/путей/прямоугольников.
- AC-3 Профиль со `schemaVersion: 1` читается после добавления полей v2.

## Тест-план

TC-0601-01, TC-0601-02.

## Риски

Расширение модели потребует версий — сразу заложить `schemaVersion`.
