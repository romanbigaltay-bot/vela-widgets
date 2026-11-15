# Vela

Виджетный слой рабочего стола Windows 11: живой фон (фото / видео / 2D-3D сцена),
бесконечный холст с виджетами, тёмная и светлая темы, стеклянные панели.

Этот каталог — реализация в монорепозитории `vela-vijets`. План,
спецификации и ТЗ живут рядом, в [`../vela-plan`](../vela-plan/PLAN.md);
репозиторий следует им и не расходится с ними: любое поведение, которого нет
в плане, сначала попадает в план.

## Состояние

| Метрика | Значение |
|---|---|
| Этап | **P2 «Оболочка» — в работе**; T-0201 закрыта, следующая — T-0202 |
| Закрыто задач | **P1 4/4** (T-0101…T-0104); **P2 1/5** (T-0201) |
| Gate G1 | **пройден**: 7 из 7 критериев подтверждены |
| Rust | 6 крейтов/бинарей; 148 тестов прошли 2026-11-15 (`--all-features`) |
| Tauri | **2.12.5**: трей, single-instance, скрытая UI-поверхность 800×600 |
| Frontend | Vite + React + TS; токены, темы, `ThemeService`, `Glass`, ui-kit, `SurfacesPanel`; 76 тестов прошли 2026-11-15 |
| CI | jobs `rust`, `coverage`, `ui`, `ipc-contracts`, `desktop`; run 42156600633 (`main`) — success, покрытие 93.68 %; PR #2 (run 43697165125) и post-merge `main` (run 43697927270) — все 5 job'ов success |
| Коммиты | T-0201 слита в `main` как `efcae65` (squash [PR #2](https://github.com/romanbigaltay-bot/vela-widgets/pull/2)), до неё `4b560e0` (PR #1) |

Подробный статус, включая расхождение плана и факта, —
[`docs/STATUS.md`](docs/STATUS.md).

## Требования

- Windows 11 22H2+ (для запуска приложения). Сборка и тесты идут и на Linux.
- Rust 1.80+ (фиксируется `rust-toolchain.toml`).
- Node.js 20+ и pnpm 9+.
- WebView2 Runtime — на Windows 11 предустановлен.

## Быстрый старт

Из корня репозитория:

```bash
# 1. Зависимости фронтенда
pnpm --dir vela-app/ui install

# 2. Rust-тесты (workspace vela-app)
cargo test --manifest-path vela-app/Cargo.toml --workspace

# 3. Разработка: Vite-сервер + приложение (трей и скрытое окно 800×600)
pnpm --dir vela-app/ui dev            # терминал 1
cargo run --manifest-path vela-app/Cargo.toml -p vela   # терминал 2

# 4. Витрина токенов (отдельно, в браузере)
# http://localhost:5173/_kit
```

Приложение стартует в трее: «Показать виджеты» открывает окно, Alt+F4
скрывает его, «Выход» завершает процесс. Без `pnpm dev` соберите UI и
включите `custom-protocol`, чтобы окно грузило вшитые ассеты:

```bash
pnpm --dir vela-app/ui build
cargo build --release --manifest-path vela-app/Cargo.toml -p vela --features tauri/custom-protocol
```

Полный набор проверок — в CI (`.github/workflows/ci.yml`) и в `AGENTS.md` §5:
`cargo fmt`, `clippy`, `arch-check`, `pnpm typecheck`, `lint`, `test`,
`format:check`, `ipc:check`.

## Структура

```
vela-app/
├── apps/desktop/          бинарь приложения: Tauri, трей, поверхности
├── crates/
│   ├── vela-core/         домен: БД, миграции, репозитории (без tauri/windows)
│   │   └── migrations/    SQL-файлы, вшитые через include_str!
│   ├── vela-ipc/          контракты Rust↔UI + кодоген (без tauri/windows)
│   │   └── contract.json  манифест каталога контрактов
│   ├── vela-platform/     Win32: окна, стили; WorkerW и захват — дальше
│   └── vela-cron/         парсер и планировщик cron       (пока заглушка)
├── ui/                    Vite + React + TS
│   ├── src/shared/api/    сгенерированные типы (не редактировать руками)
│   ├── src/shared/theme/  ThemeService и useTheme
│   ├── src/shared/ui/     Glass и минимальный ui-kit
│   └── src/styles/        токены, типографика, стекло, ui-kit
├── scripts/               arch-check.sh и прочие проверки
└── docs/                  STATUS.md (живой статус), SESSION.md (срез)
```

CI лежит не здесь, а в корне монорепозитория:
`../.github/workflows/ci.yml` (команды с `working-directory: vela-app`).

## Границы крейтов

Правило одно: **зависимость идёт только вниз, и платформа не протекает в домен.**

| Крейт | Запрещено | Почему |
|---|---|---|
| `vela-core` | `tauri`, `windows`, другие крейты проекта | домен должен тестироваться без графической среды |
| `vela-ipc` | `tauri`, `windows`, `winit`, `wgpu` | контракты нужны и в тестах, и в кодогенерации без окна |
| `vela-platform` | `tauri` | Win32-обёртки тестируются отдельно от UI-стека |

Проверяется дважды: тестами (`crates/*/tests/*rules.rs`) и скриптом
`scripts/arch-check.sh`, который читает манифесты и потому работает без сети
и без сборки.

## Соглашения

- Коммиты: `vela: [ST-XXXX-NN] краткое описание`.
- Один PR — одна подзадача из плана.
- `unwrap()` в рантайме запрещён линтом; в тестах разрешён явным
  `#![cfg_attr(test, allow(clippy::unwrap_used))]`.
- Сгенерированные файлы в `ui/src/shared/api/` не редактируются руками:
  их пишет `pnpm ipc:gen` из Rust-типов, дифф проверяется в CI.

## Известные отклонения от плана

Ведётся в [`docs/STATUS.md`](docs/STATUS.md), раздел «Отклонения».
Кратко: `cargo run` — dev-режим Tauri и требует `pnpm dev` (автономный бинарь —
`--features tauri/custom-protocol`, D-27); тесты собраны в integration-файлы
вместо unit-блоков в каждом модуле; `zod`-валидация отложена (D-18).