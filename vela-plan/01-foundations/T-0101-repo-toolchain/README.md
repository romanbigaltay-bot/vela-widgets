---
id: T-0101
stage: P1
title: Репозиторий, workspace, toolchain, CI
status: ready
est: 1.0d
owner: both
depends: []
gate: G1
---

# T-0101 — Репозиторий, workspace, toolchain, CI

## 1. Цель и границы

Создать структуру репозитория, в которой Rust-крейты и TS-фронтенд живут рядом,
собираются одной командой и проверяются CI. Никакого продуктового кода.

**Входит:** cargo workspace, Vite+React+TS, линты, форматирование, CI, скрипты
запуска, заготовка Tauri-конфига, структура каталогов.
**Не входит:** бизнес-логика, окна, стили (это T-0103).

## 2. Решение

```
vela/
├── apps/desktop/            # Tauri-бинарь: tauri.conf.json, main.rs
├── crates/
│   ├── vela-core/           # домен: типы, БД, миграции
│   ├── vela-platform/       # Win32: окна, WorkerW, захват, питание
│   ├── vela-cron/           # парсер и планировщик (P7)
│   └── vela-ipc/            # типы команд/событий + кодоген TS
├── ui/
│   ├── src/
│   │   ├── app/             # точка входа поверхности
│   │   ├── features/        # доменные папки (canvas, widgets, scene)
│   │   ├── shared/          # ui-kit, hooks, lib
│   │   └── styles/          # tokens, base, glass
│   └── package.json
├── bench/                   # скрипты замеров (PS1/Rust)
└── docs/                    # ссылка на этот план
```

Дерево крейтов выбрано так, чтобы `vela-platform` (самый хрупкий код) не
содержал бизнес-логику и мог тестироваться отдельно.

## 3. Контракты

- `pnpm dev` — Vite на 5173, Tauri смотрит на него.
- `pnpm tauri:dev` — сборка Rust + запуск.
- `pnpm ipc:gen` — см. T-0102.
- `pnpm check` — lint + typecheck + unit.
- Названия пакетов: `vela`, `vela-core`, `vela-platform`, `vela-cron`, `vela-ipc`.

## 4. Состав подзадач

| ID | Подзадача | Дней |
|---|---|---|
| ST-0101-01 | Cargo workspace и структура крейтов | 0.3 |
| ST-0101-02 | Vite + React + TS + строгий tsconfig | 0.3 |
| ST-0101-03 | Линты, форматирование, pre-commit | 0.2 |
| ST-0101-04 | CI-пайплайн и скрипты | 0.2 |

## 5. Критерии приёмки

- **AC-1** `cargo build --workspace` собирается с нуля на чистой машине за ≤ 4 мин.
- **AC-2** `pnpm install && pnpm dev` поднимает страницу с `Hello Vela`.
- **AC-3** `cargo clippy --all-targets -- -D warnings` — без замечаний.
- **AC-4** `pnpm check` проходит локально и в CI.
- **AC-5** `.editorconfig`, `.gitignore` (target, node_modules, dist, *.db) на месте.
- **AC-6** Ветка `main` защищена, CI обязателен (настраивается в GitHub settings,
  фиксируется скриншотом в PR).

## 6. Тест-план

| ID | Проверка |
|---|---|
| TC-0101-01 | Клонировать в чистый каталог, выполнить README-инструкции, всё запускается |
| TC-0101-02 | Изменить файл с ошибкой линта → CI красный |
| TC-0101-03 | `cargo test --workspace` на пустом проекте — зелёный |

## 7. Риски

- Долгая первая сборка Tauri на Windows (нужен VS Build Tools) → в README
  отдельный раздел с требованиями и проверкой `link.exe`.
- Расхождение версий Rust у разработчиков → `rust-toolchain.toml` фиксирует версию.
