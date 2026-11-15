# Статус реализации

Снимок на **2026-11-15**. Источник истины по объёму — `vela-plan/PLAN.md`;
этот файл отвечает на вопрос «что реально сделано». Проверки прогнаны локально
на Windows-GNU (Rust 1.99.1, MinGW GCC 16.2.0, pnpm 9.15.9). P1 закрыт
полностью (4 из 4); в P2 закрыта T-0201, CI run 43697165125 (PR #2) — все
5 job'ов зелёные, включая `desktop` на windows-latest.

| | |
|---|---|
| Дата проверки | 2026-11-15 |
| Репозиторий | `vela-vijets`, remote [`romanbigaltay-bot/vela-widgets`](https://github.com/romanbigaltay-bot/vela-widgets) (public); `main` — `4b560e0` (merge PR #1); рабочая ветка — `feat/t-0201-tauri-app` ([PR #2](https://github.com/romanbigaltay-bot/vela-widgets/pull/2)), коммиты `256dfd5`, `e6fbb62`, `906f06a`; CI run 43697165125 — success |
| Текущий этап | **P2 «Оболочка» — в работе; T-0201 закрыта, следующие T-0202/T-0203** |
| Gate G1 | **пройден 2026-11-06** |
| Gate G2 | **не пройден**: закрыта только T-0201 |
| Задач P1 закрыто | **4 из 4** |
| Задач P2 закрыто | **1 из 5** |

## Прогресс по этапам

| Этап | Задач | Закрыто | Статус |
|---|---|---|---|
| P1 Каркас и контракты | 4 | 4 | T-0101…T-0104 — done; T-0102 закрыта 2026-11-15 после первой реальной команды |
| P2 Оболочка | 5 | 1 | T-0201 — done; T-0202/T-0203 — следующие |
| P3 Холст | 5 | 0 | не начат |
| P4 Рантайм виджетов | 5 | 0 | не начат |
| P5 Захват | 4 | 0 | не начат |
| P6 Живой фон | 5 | 0 | не начат |
| P7 Модули | 6 | 0 | не начат |
| P8 Релиз | 5 | 0 | не начат |

## Детально по P1

| Задача | Статус | Что есть | Что осталось |
|---|---|---|---|
| T-0101 Репозиторий, workspace, CI | **done** | 5 крейтов, Vite+React+TS, fmt/clippy/eslint/prettier, arch-check, CI в корне; git и remote `romanbigaltay-bot/vela-widgets` (public); CI run 42152167354 и 42154029664 зелёные; AC-1 подтверждён (job `rust` 1 мин 44 с), AC-2 — `pnpm dev` + headless Chromium (`/`, `/_kit`, шрифт Inter Tight loaded); AC-6 — branch protection `main` (обязательные `rust`, `coverage`, `ui`, `ipc-contracts`, `strict=true`, `enforce_admins=true`) | — (закрыта 2026-11-06) |
| T-0102 Контракты IPC | **done** (2026-11-15) | `vela-ipc`: типы и кодоген; `client.ts`, `events.ts`, `transport.ts`, `useEvent.ts`; `contract.json` читают Rust-тесты и `ipc-gen.mjs`; `ipc:check` и `tsc` зелёные; AC-2 (CI) и AC-4 (CDP: `ipc probe.echo 0ms ok`) подтверждены; `InvokeMap` содержит `window.surfaces_status`; 2026-11-15 временное переименование поля в `SurfaceStatus.ts` уронило `tsc` 6 ошибками в 4 файлах, после `ipc:gen` снова зелено | — (закрыта 2026-11-15; `zod` остаётся отложенной, D-18) |
| T-0103 Токены и темы | **done** | `tokens.css` (9 spacing 6–64, шкала 12–56), локальный Inter Tight (0 внешних запросов), `ThemeService` + `useTheme`, `Glass`, ui-kit, dev-витрина `/_kit`; 70 UI-тестов зелёные; скриншоты обеих тем в `visual/`; TC-0103-03 проверен CDP (20 кликов); AC-2 — запись `theme-switch-240fps.mp4` (1920×1080, 240 fps, 1200 кадров, 0 белых кадров) | — (закрыта 2026-11-06; P2-follow-ups: живой WebView2, IPC/SQLite-синхронизация темы, D-20) |
| T-0104 SQLite | **done** | `db::open` с PRAGMA, `quick_check`, карантин; миграции, схема v1 (8 таблиц + `schema_migrations`), `ElementsRepo`/`SettingsRepo`, `apply_patch` одной транзакцией; фикстура `v0_empty.db` и тест `tc_0104_02`; AC-1 измерен (медиана 10.28 мс при пороге 50 мс), AC-4 проверен вручную (порча → бэкап → новая БД, exit 0); CI run 42152167354 — все job'ы зелёные, покрытие 93.68 % | — (закрыта 2026-11-06; 4 репозитория отложены, D-13) |

## Детально по P2

| Задача | Статус | Что есть | Что осталось |
|---|---|---|---|
| T-0201 Приложение Tauri | **done** (2026-11-15) | Tauri 2.12.5, трей, single-instance, `SurfaceManager`, UI 800×600 скрытой, `window.surfaces_status`, событие `window:surfaces_changed`, Alt+F4 скрывает, выход через трей; 148 Rust + 76 UI тестов | — (закрыта; отклонения D-21…D-27) |
| T-0202 Поверхность A (сцена) | planned | — | WorkerW, геометрия/DPI, восстановление после Explorer |
| T-0203 Поверхность B (UI) | planned | — | прозрачность, hit-regions, режимы ввода |
| T-0204 Автозапуск и питание | planned | — | autostart, power policy, session events |
| T-0205 Поверхность C и хоткеи | planned | — | overlay lifecycle, глобальные хоткеи |

## Gate G1: критерии

Оконный критерий из первой редакции («окно 800×600») перенесён в G2: Tauri
подключён в P2 (`T-0201`, решение D-16). В G1 7 критериев.

| # | Критерий | Состояние |
|---|---|---|
| 1 | `pnpm dev` показывает страницу с токенами и шрифтом | ✓ подтверждено 2026-11-06: `/` и `/_kit` — HTTP 200, headless Chromium отрендерил заголовок, health-строки и витрину; Inter Tight `status: loaded`; скриншоты обеих тем есть |
| 2 | `pnpm ipc:gen` идемпотентна, дифф пустой | ✓ подтверждено 2026-11-03 и 2026-11-06: `pnpm ipc:check` — «типы актуальны» |
| 3 | `cargo test` и `pnpm test` зелёные, покрытие ≥ 60 % | ✓ подтверждено 2026-11-06: 123 Rust + 70 UI в CI; покрытие строк 93.68 % (порог 60 %) |
| 4 | CI-пайплайн P1 проходит на push | ✓ подтверждено 2026-11-06: run [42152167354](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42152167354) и [42154029664](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42154029664), по 4 job'а success |
| 5 | БД создаётся при старте, миграция идемпотентна | ✓ подтверждено 2026-11-03: тесты + `cargo run -p vela` (exit 0, `state.db` создан) |
| 6 | Переключение `data-theme` меняет все токены без вспышек | ✓ подтверждено 2026-11-06: запись `theme-switch-240fps.mp4` (1920×1080, 240 fps CFR, 1200 кадров); средний RGB не превышает 243/241/245, кадров белее светлой темы — 0; плюс 20 переключений через CDP |
| 7 | README объясняет запуск за 3 команды | ✓ подтверждено 2026-11-06: в `vela-app/README.md` три команды из корня — `pnpm --dir vela-app/ui install`, `cargo test --manifest-path vela-app/Cargo.toml --workspace`, `pnpm --dir vela-app/ui dev` |

**Итог:** все 7 критериев подтверждены. G1 пройден 2026-11-06.

## Проверки

### Прогнано 2026-11-05 (UI, Windows)

| Проверка | Результат |
|---|---|
| `pnpm typecheck` | ✓ |
| `pnpm lint` | ✓ 0 предупреждений |
| `pnpm test` | ✓ 70/70 в 11 файлах |
| `pnpm format:check` | ✓ |
| `pnpm build` | ✓ 38 модулей; `KitPage` вырезана из production (было 49 с статическим импортом) |
| `pnpm dev` | ✓ Vite на 5173; `/` и `/_kit` отвечали 200; скриншоты обеих тем сняты в headless Edge |
| `pnpm test` (новые файлы) | ✓ `tokens` 6, `theme` 10, `theme-boot` 6, `Glass` 3, `Button` 4, `Switch` 4, `KitPage` 3 |
| `tokens.test.ts` контраст | ✓ светлая тема: `accent-text` 4.78, `text-on-accent` 4.61 |

### Прогнано 2026-11-06 (CI, GitHub Actions)

| Проверка | Результат |
|---|---|
| run [42152167354](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42152167354) | ✓ success, 2026-11-06T02:43:54Z–02:45:40Z |
| run [42154029664](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42154029664) на `89147eb` | ✓ success, 4 job'а |
| job `rust` | ✓ fmt, clippy, test, arch-check |
| job `coverage` | ✓ `--fail-under-lines 60`; строки 93.68 %, функции 94.42 % |
| job `ui` | ✓ typecheck, lint, format, test |
| job `ipc-contracts` | ✓ `pnpm ipc:check` |

### Прогнано 2026-11-06 (Rust, Windows-GNU)

| Проверка | Результат |
|---|---|
| `cargo fmt --all --check` | ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✓ без замечаний |
| `cargo test --workspace` | ✓ 118 тестов |
| `cargo test --workspace --all-features` | ✓ 123 теста |
| `bash scripts/arch-check.sh` | ✓ `arch-check: ok` |
| `cargo check -p vela-core --example bench_startup` | ✓ пример AC-1 собирается |

### Прогнано 2026-11-06 (UI, CDP)

| Проверка | Результат |
|---|---|
| `pnpm dev` + headless Chromium | ✓ `/` — HTTP 200, `<h1>Vela</h1>`, health-строки; `/_kit` — витрина; Inter Tight `status: loaded` |
| AC-4 T-0102 (CDP) | ✓ в dev-консоли `ipc probe.echo 0ms ok` |
| TC-0103-03 (CDP) | ✓ 20 переключений: тема чередуется, `<style>`/DOM/stylesheet не растут, ошибок 0 |
| AC-2 T-0103 (запись 240 fps) | ✓ `theme-switch-240fps.mp4` — 1920×1080, 240 fps CFR, 1200 кадров; максимум среднего RGB 243/241/245, кадров белее светлой темы 0; протокол `visual/theme-switch-240fps.md` |
| ST-0103-01 AC-2 (Network) | ✓ 34 запроса, все к `localhost:5173` + один `data:`-URI; внешних хостов 0; Inter Tight `status: loaded` |
| `pnpm ipc:check` | ✓ 5 тестов, «типы актуальны» |

### Прогнано 2026-11-15 (T-0201, Windows-GNU)

| Проверка | Результат |
|---|---|
| `cargo test --workspace --all-features` | ✓ 148 тестов (включая 13 `vela-platform` и 12 `vela`) |
| `cargo fmt --all --check` | ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✓ без замечаний |
| `pnpm typecheck` | ✓ |
| `pnpm lint` | ✓ 0 предупреждений |
| `pnpm test` | ✓ 76/76 в 12 файлах |
| `pnpm format:check` | ✓ |
| `pnpm ipc:check` | ✓ 6 контрактных тестов, «типы актуальны» |
| `bash scripts/arch-check.sh` | ✓ `arch-check: ok` |
| Release: старт | ✓ UI-окно создаётся скрытым за 84–169 мс, трей появляется за 656–692 мс (3 прогона, прямой `CreateProcess`) |
| Release: второй экземпляр | ✓ 44.7–45.4 мс, exit 0; первое окно остаётся видимым, число окон не растёт |
| Release: переключение трея | ✓ 6 переключений: медиана 21.71 мс, максимум 35.94 мс при пороге 100 мс |
| Release: «Выход» | ✓ exit 0, `Get-Process vela` — 0 процессов |
| Release: Alt+F4 | ✓ реальный `SendKeys('%{F4}')`: окно скрылось, процесс жив, ex-style `0x08000190` не изменился |
| Release: стили окна | ✓ `GWL_EXSTYLE = 0x08000190` (NOACTIVATE, TOOLWINDOW, WINDOWEDGE, ACCEPTFILES), `WS_EX_APPWINDOW` отсутствует; в панели задач и Alt+Tab окна нет |
| T-0102 AC-2 | ✓ временное переименование `kind` в `SurfaceStatus.ts` уронило `tsc` 6 ошибками в 4 файлах; после `pnpm ipc:gen` зелено |
| Ручной смоук T-0201 | ✓ подтверждён 2026-11-09: трей, показ/скрытие, панель поверхностей, Alt+F4, второй запуск, «Выход» |
| CI PR #2 | ✓ run 43697165125: `rust`, `coverage`, `ui`, `ipc-contracts`, `desktop` — success; `desktop` впервые на windows-latest |

### Прогнано 2026-11-03 (Rust, Windows-GNU)

| Проверка | Результат |
|---|---|
| `cargo fmt --all --check` | ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✓ без замечаний |
| `cargo test --workspace` | ✓ 117 тестов |
| `cargo test --workspace --all-features` | ✓ 122 теста (включая 5 контрактных) |
| `pnpm ipc:check` | ✓ 5 тестов, «типы актуальны» |
| `bash scripts/arch-check.sh` | ✓ `arch-check: ok` |
| `cargo run -p vela` | ✓ exit 0, `%LOCALAPPDATA%\Vela\state.db` создан |
| `pnpm typecheck` | ✓ |
| `pnpm lint` | ✓ 0 предупреждений |
| `pnpm format:check` | ✓ |
| `pnpm test` | ✓ 34/34 (до T-0103) |
| `pnpm build` | ✓ Vite production build |

Статических `#[test]` — 124; один (`uses_xdg_or_home_on_unix`) не компилируется
на Windows, поэтому локально доступны 123 (с контрактными тестами) и 118 без фич.

### Проверено чтением и git

| Что | Факт |
|---|---|
| Git | Монорепозиторий в корне `vela-vijets`, remote `romanbigaltay-bot/vela-widgets`; `main` — `4b560e0`, рабочая ветка `feat/t-0201-tauri-app` ([PR #2](https://github.com/romanbigaltay-bot/vela-widgets/pull/2)), коммиты `256dfd5`, `e6fbb62`, `906f06a` |
| БД | В `0001_init.sql` 8 таблиц; девятая, `schema_migrations`, создаётся bootstrap-DDL в `migrations.rs` (D-9) |
| Контракты | 10 сгенерированных типов; `contract.json` читают Rust-тесты и `ipc-gen.mjs`; `InvokeMap` содержит `window.surfaces_status` |
| UI-тесты | 76 тестов в 12 файлах; T-0201 добавила `SurfacesPanel` и обновление `App`/`client` |
| Rust-тесты | 148 тестов с `--all-features`; 13 в `vela-platform`, 12 в `vela` |
| Токены | `tokens.css`, `typography.css`, `glass.css`, `ui-kit.css`; hex/rgb/hsl только в `tokens.css` |
| Тема | inline-скрипт в `index.html`; `ThemeService` + `useTheme`; в P1 — `localStorage`, в P2 — SQLite/IPC (D-20) |
| CI | Root `.github/workflows/ci.yml`; run 42152167354 и 42154029664 (2026-11-06) — success, 4 job'а |
| Защита `main` | branch protection: `strict=true`, контексты `rust`, `coverage`, `ui`, `ipc-contracts`, `enforce_admins=true`; force-push и удаление запрещены |
| T-0104 AC-1 | `examples/bench_startup.rs`, 20 прогонов: медиана 10.28 мс при пороге 50 мс |
| T-0104 AC-4 | ручной прогон: мусорный `state.db` → бэкап `state.db.corrupt-*` (SHA-256 совпал), новая БД, exit 0 |
| T-0201 AC-1 | release: UI-окно скрыто за 84–169 мс, трей за 656–692 мс (3 прогона) |
| T-0201 AC-2 | `GWL_EXSTYLE = 0x08000190`; `WS_EX_APPWINDOW` отсутствует; в панели задач и Alt+Tab окна нет |
| T-0201 AC-3 | временный двойной `ensure` в debug: один лог «поверхность создана» и одно окно; код откачен |
| T-0201 AC-4 | второй экземпляр 44.7–45.4 мс, exit 0; первое окно видимо |
| T-0201 AC-5 | реальный Alt+F4: окно скрылось, процесс жив |
| T-0201 AC-6 | «Выход»: exit 0, 0 процессов `vela` |
| Документы | В `vela-plan` 189 `.md`-файлов, как и записано в `PLAN.md` §9 |

### Не проверено

- Покрытие локально не измеряется: `cargo llvm-cov` падает на Windows-GNU
  (`profiler_builtins` отсутствует); в CI измерено 93.68 % строк.
- Визуального диффа скриншотов и CI visual-джобы нет (TC-0103-02);
  скриншоты сняты вручную 2026-11-05.
- P2-follow-ups T-0103: живой WebView2, холодный старт/системная тема
  (ST-0103-02), IPC/SQLite-синхронизация темы (D-20).
- Windows-джоба `desktop` впервые прошла в PR #2: run 43697165125, все
  5 job'ов success; локально clippy/build/test тоже зелёные.
- NFR-замеры не воспроизводимы: `bench/` появится только в T-0802 (P8).
- Сборка под MSVC не проверялась: локально стоит GNU-toolchain.
- `--safe-mode` пока только разбирается; содержательная проверка на сломанном
  WorkerW — T-0202 (D-25).

Числа «151 тест», «0 предупреждений», «чисто» из прошлой версии статуса
заменены фактическими результатами прогонов выше.

## Отклонения от плана

| ID | Отклонение | Причина | Статус / что делать |
|---|---|---|---|
| D-1 | `tauri` не подключён в P1 | G1 не должен зависеть от графического стека | **закрыто 2026-11-15**: Tauri 2.12.5 подключён в T-0201 |
| D-2 | Тесты лежат в `tests/`, а не unit-блоками в модулях | архитектурным проверкам нужен доступ снаружи крейта | принято |
| D-3 | В `vela-core` был только `health()` | P1 закрывает контракты, домен наполняется в T-0104 | закрыто в T-0104 |
| D-4 | `rusqlite` не использовался | — | закрыто в ST-0104-01 |
| D-5 | 5 токенов вместо полного набора | T-0103 не начат | **закрыто 2026-11-05**: полный набор в `tokens.css` |
| D-6 | CI-джоба `ipc-contracts` требует `cargo` | `ts-rs` генерирует из Rust | принято |
| D-7 | В ST-0104-01 был `spawn_blocking`, в коде — поток-владелец на `mpsc` | нельзя тянуть tokio в домен | **документы исправлены 2026-11-02**; код соответствует |
| D-8 | `Db::call` принимает замыкание | в P1 известно только «открыть/закрыть» | принято; обёртки появятся с потребителем |
| D-9 | `schema_migrations` создаётся bootstrap-DDL | иначе замкнутый круг при чтении реестра | принято, единственный DDL вне миграций |
| D-10 | Транзакция на файл, а не на весь прогон | частичный прогресс консистентен, видно место падения | принято; rollback файла ≠ down-миграция |
| D-11 | `elements.kind` без CHECK, `monitor_id` без FK | CHECK дорого менять, `monitors` — не справочник | принято; валидация в домене |
| D-12 | `migrate_with` публичный ради тестов | иначе AC-3 не проверить одной миграцией | принято |
| D-13 | `ProfilesRepo`, `NotesRepo`, `CronRepo`, `CaptureRepo` не созданы | у таблиц ещё нет потребителя | принято; создавать в T-0305, ST-0404-01, ST-0704-02, T-0504 |
| D-14 | Неизвестный `kind` читается как `Widget` | элемент из новой версии не должен исчезать | открыто; добавить `ElementKind::Unknown` в ST-0302-01 |
| D-15 | `apply_patch` возвращает только затронутые элементы | полный снимок — лишний трафик на каждый жест | принято; doc-комментарии исправлены |
| D-16 | G1 требовал окно Tauri 800×600 | в P1 Tauri намеренно нет | **закрыто 2026-11-02**: критерий перенесён в G2, в G1 осталось 7 критериев |
| D-17 | Покрытие ≥ 60 % не измерено | Windows-GNU без profiler runtime; job в CI добавлен | **закрыто 2026-11-06**: 93.68 % строк в CI |
| D-18 | Dev-валидация `zod` не реализована | в P1 нет транспорта и реальных команд | отложено до P2; ТЗ ST-0102-03 обновлено |
| D-19 | Репозиторий не был инициализирован | граница репозитория была не выбрана | **закрыто 2026-11-02**: монорепозиторий `vela-vijets`, CI в корне |
| D-20 | Тема в P1 хранится в `localStorage`, а не в SQLite | в P1 не было Tauri, типизированной команды и `app.theme_changed` | принято; SQLite-синхронизация — ST-0103-02, `setThemeFromHost` готов, inline-скрипт останется |
| D-21 | Меню трея сокращено до «Показать виджеты» и «Выход» | сцена/режим/профиль/настройки появятся вместе с задачами T-0202…T-0205; неработающий пункт хуже отсутствующего | принято; расширять по мере появления потребителей |
| D-22 | Тост «Vela уже запущена» не показан | инфраструктуры тостов ещё нет; второй экземпляр просто показывает UI первого | принято; вернуться вместе с настройками/уведомлениями |
| D-23 | Событие называется `window:surfaces_changed`, а не `window.surfaces_changed` | Tauri 2 разрешает в именах событий только буквы, цифры, `-`, `/`, `:` и `_`; точка отклоняется | принято; план и UI обновлены |
| D-24 | UI-поверхность создаётся с `transparent: false`, `parent_to_workerw` возвращает `Unsupported` | прозрачность и hit-regions — T-0203; WorkerW — T-0202 | принято; не маскировать: неподдерживаемый флаг даёт ошибку |
| D-25 | `--settings`, `--no-scene`, `--safe-mode` разбираются, но потребителей нет; содержательная проверка `--safe-mode` невозможна | настройки — T-0204, сцена/WorkerW — T-0202 | принято; флаги сохранены в `CliOptions`, проверка перенесена в T-0202 |
| D-26 | Windows-джоба `desktop` добавлена в CI, покрытие по-прежнему считается на Linux | Win32/Tauri-код не компилируется на Linux; покрытие Windows-кода локально не измеримо | принято; `desktop` проверяет clippy/build/test, `coverage` остаётся Linux; первый зелёный прогон — PR #2, run 43697165125 |
| D-27 | `cargo run -p vela` — dev-режим Tauri и грузит `devUrl`; автономный бинарь требует `--features tauri/custom-protocol` | без Vite-сервера окно пустое; release-смоук T-0201 проверял окно/трей, а не контент | принято; README описывает оба режима, `--all-features` не включает `custom-protocol` |

## Известные дефекты

| # | Дефект | Влияние | Статус |
|---|---|---|---|
| B-1 | Prettier переформатировал `src/shared/api/*.ts` | сгенерированные типы расходились с Rust | исправлено: `.prettierignore` |
| B-2 | `rustfmt` с `use_small_heuristics = "Max"` | нечитаемые диффы | исправлено |
| B-3 | `unwrap_used = deny` ломал тесты | нельзя было включить линт | исправлено: исключение для тестов |
| B-4 | ESLint не знал про `process`/`console` в `scripts/*.mjs` | 18 ошибок линта | исправлено: отдельный блок `globals.node` |
| B-5 | Не было `clippy.toml`, `rustfmt.toml`, workspace-линтов | правила не применялись | исправлено |
| B-6 | Не было git-репозитория | нет истории | **исправлено 2026-11-02** |
| B-7 | `languageOptions.globals` заменял набор глобалов | UI-файлы теряли `window`/`document` | исправлено |
| B-8 | `.editorconfig` задавал 4 пробела для `.mjs` | Prettier и ESLint расходились | исправлено |
| B-9 | Успешные IPC-вызовы логировались как `warn` | вытесняли ошибки из буфера | исправлено: уровень `debug` |
| B-10 | `index.ts` был заявлен, но не существовал | точка входа контрактов была фикцией | исправлено |
| B-11 | Относительный `TS_RS_EXPORT_DIR` создавал мусор | копия контракта выглядела настоящей | исправлено |
| B-12 | Список рукописных файлов был хардкодом | расходился с манифестом | исправлено: читается из `contract.json` |
| B-13 | Новый тест-файл не попал в `contract.json` | падал `contract_sync` | исправлено |
| B-14 | Документы заявляли 8 коммитов и `5121c3e` | истории не существовало | **исправлено 2026-11-02** |
| B-15 | CI лежал внутри `vela-app` | при корне `vela-vijets` GitHub его не видел | **исправлено 2026-11-02** |
| B-16 | ESLint игнорировал `*.gen.ts`, которых нет | сгенерированные `*.ts` линтовались | **исправлено 2026-11-02** |
| B-17 | `pnpm size` вызывал несуществующий скрипт | команда падала | **исправлено 2026-11-02**: скрипт удалён |
| B-18 | `contract.json` обещал чтение `ipc-gen.mjs`, но скрипт его не читал | два источника истины | **исправлено 2026-11-02** |
| B-19 | Корневые `STATUS.md`/`SESSION.md` дублировали `vela-app/docs/` | копии могли разойтись | **исправлено 2026-11-02**: оставлены ссылки |
| B-20 | `PLAN.md` заявлял 188 документов | фактически 189 | **исправлено 2026-11-02** |
| B-21 | `cargo test --all-features` падал без `TS_RS_EXPORT_DIR` | контентные тесты писали в реальный каталог и требовали env | **исправлено 2026-11-03**: временные каталоги; stray-check без env пропускается |
| B-22 | `import.meta.url` под Vitest оказался http-URL | `fileURLToPath` падал: «The URL must be of scheme file» | **исправлено 2026-11-05**: путь от `process.cwd()` с двумя кандидатами |
| B-23 | Глобальный `localStorage` Node 25 без файла не имел `clear` | 13 тестов темы падали | **исправлено 2026-11-05**: in-memory полифилл в `vitest.setup.ts` |
| B-24 | В светлой теме `--accent` как текст давал 4.11:1 | ссылки и значения не проходили AA | **исправлено 2026-11-05**: `--accent-text` 4.78:1; `--text-on-accent` 4.61:1 |
| B-25 | `KitPage` импортировалась статически | dev-витрина попадала в production-бандл | **исправлено 2026-11-05**: lazy + `import.meta.env.DEV`, 38 модулей вместо 49 |
| B-26 | `pnpm ipc:gen` в P1 ничего не писала, а `--check` менял дерево | кодогенерация была фикцией, проверка могла «починить» расхождение | **исправлено 2026-11-15**: экспорт во временный каталог, сравнение, запись только в режиме генерации |
| B-27 | `WS_EX_APPWINDOW` возвращался после `window.show()` | поверхность появлялась в панели задач и Alt+Tab | **исправлено 2026-11-15**: показ через `ShowWindow`, повторное снятие `APPWINDOW` до и после активации |
| B-28 | Tauri отклонял событие `window.surfaces_changed` | событие не доходило до UI | **исправлено 2026-11-15**: `window:surfaces_changed` |
| B-29 | Тестовый exe падал с `STATUS_ENTRYPOINT_NOT_FOUND` | `TaskDialogIndirect` требует comctl32 v6, а манифест линковался только в `[[bin]]` | **исправлено 2026-11-15**: `embed-resource` линкует манифест и в тесты |
| B-30 | GDI-тест падал при параллельном создании окон | счётчик `GetGuiResources` видит окна других тестов | **исправлено 2026-11-15**: `WINDOW_TEST_LOCK` сериализует оконные тесты |
| B-31 | Замеры второго экземпляра через `ShellExecute` давали 250–400 мс | оболочка добавляла время до `CreateProcess` | **исправлено 2026-11-15**: замер прямым `CreateProcess`, 44.7–45.4 мс |

## Следующие шаги (по порядку)

1. **T-0202** — поверхность A: WorkerW, геометрия/DPI, восстановление после
   Explorer. Это следующий обязательный шаг P2.
2. **T-0203** — поверхность B: прозрачность, hit-regions, режимы ввода;
   можно вести параллельно с T-0202.
3. **T-0204** — автозапуск, питание, session events; **T-0205** — оверлей
   и глобальные хоткеи (включая `Ctrl+Alt+Q`).
4. T-0103 P2-follow-ups: живой WebView2, ST-0103-02 (холодный старт,
   системная тема, IPC/SQLite), D-20.
5. D-14 (`ElementKind::Unknown`) — в ST-0302-01, вместе с потребителем.
6. `bench/` и NFR-замеры — в T-0802 (P8).
