---
id: ST-0204-03
parent: T-0204
title: Сон, блокировка, завершение сеанса
status: ready
est: 0.5d
owner: rust
depends: [ST-0204-02]
---

# ST-0204-03 — Сеансовые события

## Цель

Ни одной потерянной записи и ни одного зависания при выходе из Windows.

## Реализация

1. `WTSRegisterSessionNotification` → `WM_WTSSESSION_CHANGE`:
   `WTS_SESSION_LOCK` → `pause_all()`, `WTS_SESSION_UNLOCK` → `resume_all()`.
2. `WM_POWERBROADCAST`: `PBT_APMSUSPEND` → `flush()` + пауза;
   `PBT_APMRESUMEAUTOMATIC` → `resume()` + `catch_up()` (хук для P7).
3. `WM_ENDSESSION` → синхронный `flush()` с таймаутом 800 мс, затем выход;
   если не успели — записать в лог «unclean shutdown», данные в БД защищены WAL.
4. Центральный `LifecycleService` с состоянием `Running|Suspended|Locked|Exiting`,
   идемпотентные `pause/resume`.
5. Все таймеры (виджеты, сцена) подписаны на это состояние — не на сырые сообщения.

## Критерии приёмки

- AC-1 Блокировка экрана → сцена и анимации стоят (0 fps).
- AC-2 Сон 30 мин → пробуждение: часы корректны, таймеры не «прыгнули» дважды.
- AC-3 Завершение сеанса при активном перетаскивании не теряет последний патч.
- AC-4 Выход по `WM_ENDSESSION` ≤ 1 с.

## Тест-план

TC-0204-04 + проверка логов на «unclean shutdown» при принудительном завершении.

## Риски

`WM_ENDSESSION` может не прийти при жёстком выключении — WAL защищает данные,
но последний патч (≤ 250 мс) может потеряться; это документируется.
