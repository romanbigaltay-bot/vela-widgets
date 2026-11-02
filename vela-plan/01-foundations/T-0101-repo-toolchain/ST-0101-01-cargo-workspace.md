---
id: ST-0101-01
parent: T-0101
title: Cargo workspace и структура крейтов
status: ready
est: 0.3d
owner: rust
depends: []
---

# ST-0101-01 — Cargo workspace и структура крейтов

## Цель

Рабочее пространство из пяти крейтов, которое собирается целиком и не имеет
циклических зависимостей.

## Реализация

1. `Cargo.toml` в корне: `[workspace] members = ["apps/desktop", "crates/*"]`,
   `resolver = "2"`, общие `[workspace.dependencies]` (tauri, serde, rusqlite,
   thiserror, anyhow, tracing, windows).
2. Каждый крейт — `lib.rs` с `pub fn health() -> &'static str` (заглушка, чтобы
   workspace не был пустым и тесты имели что проверять).
3. `apps/desktop/src/main.rs` — `fn main() { vela::run() }`.
4. `rust-toolchain.toml`: канал `stable`, компоненты `rustfmt`, `clippy`.
5. Граф зависимостей: `vela → vela-core, vela-platform, vela-ipc`;
   `vela-platform → vela-core`; `vela-ipc → vela-core`. Обратных стрелок нет.

## Критерии приёмки

- AC-1 `cargo build --workspace` проходит.
- AC-2 `cargo tree -p vela-core` не содержит `tauri`.
- AC-3 Добавление `vela-core` в `vela-platform` не меняет публичный API последнего.

## Тест-план

`cargo test --workspace` + один тест на `health()` в каждом крейте.

## Риски

Вынесение `tauri` в зависимость `vela-core` сделает домен непереиспользуемым —
запрещено явно, проверяется в ревью.
