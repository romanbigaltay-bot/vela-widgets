---
id: ST-0102-01
parent: T-0102
title: Крейт vela-ipc: типы, конверт, ошибки
status: done
est: 0.3d
owner: rust
---

# ST-0102-01 — Крейт vela-ipc

## Цель

Все передаваемые структуры в одном крейте, компилируемом без `tauri`.

## Реализация

1. `Cargo.toml`: `serde`, `ts-rs`, `thiserror`. Запрещено: `tauri`, `windows`.
2. `lib.rs`: модули `commands/`, `events.rs`, `errors.rs`, `envelope.rs`,
   `ids.rs` (обёртки `ElementId`, `WidgetId`, `MonitorId` — не голые строки).
3. `ids.rs`: newtype-обёртки с `Display`, `FromStr`, `Serialize`, `Deserialize`
   для всех идентификаторов. Голые `String` в публичных типах запрещены.
4. `commands/`: по одному модулю на группу; каждая команда — `#[derive(Serialize, TS)]`
   request + response.
5. `errors.rs`: `enum ErrorCode` + `VelaError { code, message, retryable }`.
6. `envelope.rs`: `Envelope<T>` с конструкторами `ok()`, `err()`.

## Критерии приёмки

- AC-1 `cargo test -p vela-ipc` — 100 % публичных типов имеют serde-тесты round-trip.
- AC-2 `cargo tree -p vela-ipc` не содержит `tauri`.
- AC-3 Все ID — newtype; попытка передать `String` не компилируется.

## Тест-план

Property-тест (proptest) на round-trip сериализации всех ID и `Envelope`.

## Доказательства (2026-11-15)

- AC-1: wire-format тесты в `vela-ipc` (включая `SurfaceKind`,
  `SurfaceState`, `SurfaceStatus`) и round-trip `Envelope` зелёные;
  `cargo test --workspace --all-features` — 148 тестов.
- AC-2: `cargo tree -p vela-ipc` не содержит `tauri`; arch-check зелёный.
- AC-3: идентификаторы — newtype-обёртки (`MonitorId`, `ElementId`,
  `WidgetId`, `ProfileId`); голых `String` в публичных типах нет.

## Риски

`ts-rs` для generic `Envelope<T>` требует явной аннотации экспорта — делается
по одному файлу на конкретный `T`, иначе TS получит `any`.
