---
id: ST-0102-02
parent: T-0102
title: Кодоген TS и CI-проверка
status: ready
est: 0.3d
owner: rust
depends: [ST-0102-01]
---

# ST-0102-02 — Кодоген TS

## Цель

Типы в TS обновляются одной командой и не расходятся с Rust.

## Реализация

1. `crates/vela-ipc/tests/export.rs`: вызов `T::export_all_to("ui/src/shared/api")`.
2. Скрипт `pnpm ipc:gen`: `cargo test -p vela-ipc --features ts-export export`.
3. В сгенерированном каталоге — `index.ts`, который реэкспортирует все типы
   и `INVOKE_MAP` (имя команды → тип request/response).
4. CI-шаг: `pnpm ipc:gen && git diff --exit-code -- ui/src/shared/api` —
   падает, если типы не перегенерированы.
5. Правило: файлы в `api/` не редактируются вручную; в шапке — комментарий.

## Критерии приёмки

- AC-1 Повторный `pnpm ipc:gen` даёт пустой дифф.
- AC-2 Переименование поля в Rust → `tsc --noEmit` падает на UI-потребителе.
- AC-3 В `INVOKE_MAP` присутствуют все команды из `commands/`.

## Тест-план

TC: изменить имя поля, запустить typecheck, ожидать ошибку; откатить.

## Риски

Порядок экспорта нестабилен → задать `BTreeMap` для реестра и сортировку вывода.
