---
id: ST-0302-01
parent: T-0302
title: Типы элементов и валидация схем
status: ready
est: 0.5d
owner: rust
---

# ST-0302-01 — Типы элементов

## Цель

`Element` с гарантиями: идентификаторы типизированы, `settings` валидируются,
ничего «сырого» в домен не попадает.

## Реализация

1. `vela-core::element`: `Element`, `ElementKind`, `Rect`, `PatchOp`, `ElementPatch`,
   `LayoutSnapshot`.
2. `Element::validate(&self, schema_registry)` — проверка: размеры > 0, `z` в
   диапазоне, `kind == Widget` требует `widget_id`, `settings` соответствует схеме.
3. `Rect` — только конечные числа, `w/h > 0`, ограничение `|x|,|y| ≤ 1e5`.
4. Сериализация: `camelCase` для TS, `serde(rename_all = "camelCase")`.
5. Тесты: property-тест «валидный элемент всегда сериализуется и обратно».

## Критерии приёмки

- AC-1 Невалидный `settings` даёт `VelaError{InvalidInput}` с путём до поля.
- AC-2 `NaN`/`Infinity` в координатах отклоняются.
- AC-3 `Element` сериализуется в TS-тип без ручной правки (ts-rs).

## Тест-план

Unit: 15 валидных и 15 невалидных кейсов; proptest на round-trip.

## Риски

Валидация по схеме на каждом патче может быть дорогой → кэш скомпилированных
схем по `widget_id`.
