---
id: ST-0104-03
parent: T-0104
title: Репозитории и тесты
status: ready
est: 0.4d
owner: rust
depends: [ST-0104-02]
---

# ST-0104-03 — Репозитории

## Цель

Типизированный доступ к данным без SQL в бизнес-логике.

## Реализация

1. Трейты: `SettingsRepo`, `ElementsRepo`, `ProfilesRepo`, `NotesRepo`,
   `CronRepo`, `CaptureRepo` в `vela-core::repo`.
2. Реализация на `rusqlite` с `params!`; все запросы — `prepare_cached`.
3. `ElementsRepo::list_for_monitor(monitor, profile)` — единственная точка чтения
   раскладки; сортировка по `z`.
4. `ElementsRepo::apply_patch(Vec<ElementPatch>)` — одна транзакция, возвращает
   полное новое состояние (для рассылки в другие поверхности).
5. Маппинг `row -> Element` в одном месте, тест на каждое поле.
6. Все временные метки — `i64` миллисекунды UTC.

## Критерии приёмки

- AC-1 `apply_patch` с 50 элементами — ≤ 15 мс на счётчике.
- AC-2 Частичный сбой в патче откатывает весь патч.
- AC-3 Ни одного `SELECT *` — только явные поля.

## Тест-план

Для каждого трейта — минимум 3 теста (пусто, одна запись, много);
property-тест на round-trip `Element`.

## Риски

`state_json`/`settings_json` без схемы → валидация на уровне домена перед
записью, чтобы не копить мусор.
