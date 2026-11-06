---
id: T-0101
stage: P1
title: Репозиторий, workspace, toolchain, CI
status: done
est: 1.0d
owner: both
depends: []
gate: G1
---

# T-0101 — Репозиторий, workspace, toolchain, CI

## 0. Фактическое состояние (2026-11-06)

Каркас создан, все AC подтверждены:

- репозиторий — монорепозиторий `vela-vijets`, remote
  `github.com/romanbigaltay-bot/vela-widgets` (public); CI перенесён в корневой
  `.github/workflows/ci.yml`, команды выполняются с `working-directory: vela-app`;
- 2026-11-06 `main` запушен, первый прогон CI
  ([run 42152167354](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42152167354))
  зелёный: `rust`, `coverage`, `ui`, `ipc-contracts` — success;
- AC-1 подтверждён этим прогоном: job `rust` на чистом раннере занял 1 мин
  44 с при пороге 4 мин;
- AC-2 подтверждён 2026-11-06: dev-сервер на 5173 отдаёт `HTTP 200`, в
  headless Chromium отрендерены заголовок `Vela`, health-строки и ссылка
  `/_kit`; литерал `Hello Vela` из первой редакции ТЗ заменён страницей
  T-0103 (см. §5);
- AC-6 подтверждён 2026-11-06: репозиторий переведён в public, branch
  protection на `main` включён через API; обязательные проверки — `rust`,
  `coverage`, `ui`, `ipc-contracts`, `strict=true`, `enforce_admins=true`;
  последний прогон 42154029664 на `89147eb` — все четыре job'а success;
- локально 2026-11-03 зелёные `cargo fmt`, `cargo clippy -D warnings`,
  `cargo test`, `scripts/arch-check.sh`, `pnpm typecheck/lint/test/format:check`;
  2026-11-06 Rust-проверки повторены (123 теста с `--all-features`);
- Tauri-конфиг и `bench/` не созданы: это P2 (`T-0201`) и P8 (`T-0802`).

Поэтому статус — `done`: все AC подтверждены.

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
├── bench/                   # замеры: появятся в P8 (T-0802), в P1 отсутствуют
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
- **AC-2** `pnpm install && pnpm dev` поднимает страницу каркаса: заголовок
  `Vela`, health-строки и ссылка на `/_kit`. (В первой редакции ТЗ был текст
  `Hello Vela`; после T-0103 страница стала витриной токенов — замена
  зафиксирована 2026-11-06, см. §8.)
- **AC-3** `cargo clippy --all-targets -- -D warnings` — без замечаний.
- **AC-4** `pnpm check` проходит локально и в CI.
- **AC-5** `.editorconfig`, `.gitignore` (target, node_modules, dist, *.db) на месте.
- **AC-6** Ветка `main` защищена, CI обязателен. **Выполнено 2026-11-06:**
  branch protection через API; обязательные проверки `rust`, `coverage`, `ui`,
  `ipc-contracts`; `strict=true`, `enforce_admins=true`.

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

## 8. Доказательства (2026-11-06)

- AC-1 — CI run 42152167354, job `rust` на чистом раннере: 1 мин 44 с при
  пороге 4 мин.
- AC-2 — `pnpm dev` на 5173; `HTTP 200`, headless Chromium отрендерил
  `<h1>Vela</h1>`, health-строки (`ui`, `theme`, `renderer`) и ссылку `/_kit`.
  Скриншоты страницы — в `T-0103-design-tokens/visual/`.
- AC-3, AC-4 — CI run 42152167354: jobs `rust` (clippy) и `ui`
  (typecheck/lint/format/test) — success.
- AC-5 — `.editorconfig` в `vela-app/`, `.gitignore` в корне и в `vela-app/`
  покрывают `target`, `node_modules`, `dist`, `*.db`.
- AC-6 — 2026-11-06 репозиторий переведён в public; `PUT
  /branches/main/protection` принят: `strict=true`, контексты `rust`,
  `coverage`, `ui`, `ipc-contracts`, `enforce_admins=true`. Проверка чтением
  API: те же значения; force-push и удаление ветки запрещены. CI run
  42154029664 на `89147eb` — все четыре job'а success.
