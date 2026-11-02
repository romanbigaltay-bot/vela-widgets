---
id: ST-0605-01
parent: T-0605
title: Политика и её применение
status: ready
est: 0.5d
owner: rust+ui
---

# ST-0605-01 — Политика

## Цель

Одна чистая функция, определяющая режим сцены, и её честное применение.

## Реализация

1. `vela-core::scene_power::mode(power, foreground_fullscreen, scene_visible,
   overlay_open, user_hidden) -> SceneMode`.
2. Приоритеты: `user_hidden` > `overlay_open` > `fullscreen` > батарея.
3. Применение в Rust → событие `scene.set_mode` в A; A останавливает цикл,
   ставит видео на паузу, сохраняет последний кадр в FBO для показа.
4. Тест: таблица 16 комбинаций.
5. Лог переходов (dev): `scene mode: active -> paused (reason: fullscreen)`.

## Критерии приёмки

- AC-1 Все 16 комбинаций дают ожидаемый режим.
- AC-2 Переход в `paused` ≤ 300 мс, кадр остаётся.
- AC-3 Возврат в `active` не вызывает рывка (видео продолжается с паузы).

## Тест-план

TC-0605-01, TC-0605-03.

## Риски

Определение fullscreen может «дрожать» на границе (Alt+Tab) → гистерезис 500 мс.
