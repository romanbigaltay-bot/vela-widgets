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
| Этап | **P1 закрыт; следующий — P2 «Оболочка»** |
| Закрыто задач | **3 из 4**: T-0101, T-0103, T-0104 — done; T-0102 — в работе (P2-часть) |
| Gate G1 | **пройден**: 7 из 7 критериев подтверждены |
| Rust | 5 крейтов; 123 теста прошли 2026-11-06 (`--all-features`), в CI — success |
| Frontend | Vite + React + TS; токены, темы, `ThemeService`, `Glass`, ui-kit; 70 тестов прошли 2026-11-05, в CI — success; 20 переключений темы проверены CDP 2026-11-06; запись переключения 240 fps — в `vela-plan/.../T-0103-design-tokens/visual/theme-switch-240fps.mp4` |
| CI | run 42154029664 (2026-11-06): `rust`, `coverage`, `ui`, `ipc-contracts` — success; покрытие строк 93.68 % |
| Коммиты | история начата 2026-11-02; последние — `e36a1ef` (T-0104), `804ff43` (статус), `0de490f` (закрытие T-0104), `89147eb` (доказательства P1) |

Подробный статус, включая расхождение плана и факта, —
[`docs/STATUS.md`](docs/STATUS.md).

## Требования

- Windows 11 22H2+ (для запуска приложения). Сборка и тесты идут и на Linux.
- Rust 1.80+ (фиксируется `rust-toolchain.toml`).
- Node.js 20+ и pnpm 9+.
- WebView2 Runtime — на Windows 11 предустановлен.

## Быстрый старт (3 команды)

Из корня репозитория:

```bash
# 1. Зависимости фронтенда
pnpm --dir vela-app/ui install

# 2. Rust-тесты (workspace vela-app)
cargo test --manifest-path vela-app/Cargo.toml --workspace

# 3. Витрина токенов
pnpm --dir vela-app/ui dev   # http://localhost:5173/_kit
```

Полный набор проверок — в CI (`.github/workflows/ci.yml`) и в `AGENTS.md` §5:
`cargo fmt`, `clippy`, `arch-check`, `pnpm typecheck`, `lint`, `test`,
`format:check`, `ipc:check`.

## Структура

```
vela-app/
├── apps/desktop/          бинарь приложения (Tauri подключается в T-0201)
├── crates/
│   ├── vela-core/         домен: БД, миграции, репозитории (без tauri/windows)
│   │   └── migrations/    SQL-файлы, вшитые через include_str!
│   ├── vela-ipc/          контракты Rust↔UI + кодоген (без tauri/windows)
│   │   └── contract.json  манифест каталога контрактов
│   ├── vela-platform/     Win32: окна, WorkerW, захват    (пока заглушка)
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
Кратко: `tauri` ещё не подключён (осознанно, это ST-0201-01); тесты собраны
в integration-файлы вместо unit-блоков в каждом модуле; `zod`-валидация
отложена до P2 (D-18).