---
id: ST-0405-01
parent: T-0405
title: Машина таймера, точность, персистентность
status: ready
est: 0.6d
owner: ui
---

# ST-0405-01 — Машина таймера

## Цель

Состояние, из которого всегда можно восстановить истину, даже после крэша.

## Реализация

1. `timerMachine.ts`: чистая функция-редьюсер `(state, event) -> state`;
   события `start/pause/resume/reset/tick/finish/restore`.
2. Состояние: `{mode, duration, endsAt, elapsed, running, completedAt, cycles}`.
3. `remaining = max(0, endsAt - now)`; по `remaining === 0` → `finish` (один раз,
   проверка `completedAt == null`).
4. Персистентность: `endsAt` и `mode` в `state_json`; восстановление при монтировании.
5. Пропущенные тики: если `now - lastTick > 2000` — считать «catch-up»,
   вызвать `finish` с флагом `missed: true`.

## Критерии приёмки

- AC-1 Все 12 переходов покрыты тестами.
- AC-2 `finish` не может произойти дважды для одного `endsAt`.
- AC-3 Восстановление из `state_json` даёт корректный остаток.

## Тест-план

TC-0405-01, TC-0405-02 (с фейковым `ClockService`).

## Риски

Опора на `Date.now()` делает тесты недетерминированными → только через
`ctx.clock`, который инжектируется.
