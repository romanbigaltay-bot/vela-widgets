---
id: ST-0104-02
parent: T-0104
title: Миграции и начальная схема
status: in-progress
est: 0.3d
owner: rust
depends: [ST-0104-01]
---

# ST-0104-02 — Миграции

## Цель

Применение SQL-файлов по порядку, с фиксацией версии и бэкапом перед изменением.

## Реализация

1. `migrations/0001_init.sql` — единственный источник схемы (без
   `IF NOT EXISTS`, чтобы ошибки были явными); в README задачи DDL не
   дублируется.
2. `migrate(db)`: читает `schema_migrations`, применяет недостающие в одной
   транзакции на файл, пишет версию.
3. Перед миграцией с версии > 0 — копия файла в `state.db.bak-<version>`.
4. Встроенные миграции через `include_str!` — приложение не зависит от внешних
   файлов после установки.
5. Тест-фикстура `migrations/fixtures/v0_empty.db` для проверки «с нуля»:
   пустой файл (0 байт), который SQLite считает валидной пустой базой.
   Исключение из правила `*.db` — в `vela-app/.gitignore`; миграцию проверяет
   `tc_0104_02_migrates_v0_empty_fixture`.

## Критерии приёмки

- AC-1 Вторая миграция не применяется повторно.
- AC-2 Миграция в середине падает → транзакция откатывается, версия не меняется.
- AC-3 Бэкап создаётся до изменения при `version > 0`.

## Тест-план

TC-0104-01, TC-0104-02 из README задачи.

## Доказательства (2026-11-05)

- `tc_0104_02_migrates_v0_empty_fixture` копирует фикстуру во временный
  каталог, открывает её как `OpenOutcome::Opened` и мигрирует с версии 0 до 1
  без бэкапа.
- AC-1 подтверждён `ac1_second_run_applies_nothing`.
- AC-2 подтверждён `ac2_failing_migration_rolls_back_and_keeps_version`.
- AC-3 подтверждён `ac3_backup_created_before_migration_when_data_exists` и
  `ac3_full_path_creates_backup_when_moving_from_v1_to_v2`.
- `cargo test --workspace --all-features` — 123 зелёных теста; `cargo fmt`,
  `cargo clippy -- -D warnings` и `scripts/arch-check.sh` зелёные.

## Риски

Забыть `PRAGMA foreign_keys` при миграции → включено на уровне подключения,
а не миграции.
