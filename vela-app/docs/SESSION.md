# Итог сессии и старт следующей

Срез на **2026-11-15**. Источник истины по объёму — `vela-plan/PLAN.md`;
живой статус — [`STATUS.md`](STATUS.md). Этот файл фиксирует контекст, чтобы
не восстанавливать его по истории коммитов. P1 закрыт полностью; в P2
закрыта T-0201, следующая — T-0202.

> **Важно.** До 2026-11-02 git-истории в проекте не было. Хеши `b7b3b2e`,
> `5121c3e` и «8 коммитов» из прежней версии этого файла никогда не
> существовали; они удалены. Реальные коммиты: `57fb5a0` (импорт), `a217cb5`
> (CI/инструменты), `d526809` (документы), `16e0cde` (coverage и контрактные
> тесты), `bfd38c8` (проверенные результаты), `c0d9fa2` (T-0103),
> `6a1ff61` (витрина и скриншоты), `412d38f` (крупная шкала), `ba1f58f`
> (скролл и отступы витрины), `b697352` (T-0103), `e36a1ef` (T-0104),
> `804ff43` (статус), `0de490f` (закрытие T-0104) и `89147eb`
> (доказательства P1).

---

## 1. Где мы в плане

**Этап P2 «Оболочка» — в работе.** T-0201 закрыта 2026-11-15: Tauri 2.12.5,
трей, single-instance, `SurfaceManager`, скрытая UI-поверхность 800×600,
первая реальная IPC-команда `window.surfaces_status`, Alt+F4 скрывает,
выход через трей. Следующие — T-0202 (WorkerW) и T-0203 (UI-поверхность);
их можно вести параллельно.

**P1 закрыт полностью (4 из 4).** T-0102 закрыта 2026-11-15: P2-часть
подтверждена первой командой и падением `tsc` при переименовании поля.
Gate G1 пройден 2026-11-06.

| Задача P1 | Дней | Состояние | Вердикт |
|---|---|---|---|
| T-0101 Репозиторий, workspace, CI | 1.0 | каркас, git, remote `romanbigaltay-bot/vela-widgets` (public), CI run 42154029664 зелёный; AC-1 подтверждён (job `rust` 1 мин 44 с), AC-2 — `pnpm dev` + headless Chromium; AC-6 — branch protection `main` (`strict=true`, `rust`/`coverage`/`ui`/`ipc-contracts`, `enforce_admins=true`) | **done** (2026-11-06) |
| T-0102 Контракты IPC | 1.0 | типы, клиент, события, `useEvent`; `ipc:check` зелёный; AC-2 (CI) и AC-4 (CDP: `ipc probe.echo 0ms ok`) подтверждены; `InvokeMap` содержит `window.surfaces_status`; 2026-11-15 переименование поля уронило `tsc` 6 ошибками, после `ipc:gen` зелено | **done** (2026-11-15); `zod` остаётся отложенной (D-18) |
| T-0103 Токены, темы, ThemeService | 1.0 | токены (spacing 6–64, шкала 12–56), `ThemeService`, `Glass`, ui-kit, `/_kit`; 70 UI-тестов; скриншоты обеих тем; TC-0103-03 — 20 переключений через CDP; AC-2 — запись `theme-switch-240fps.mp4` (1920×1080, 240 fps, 1200 кадров, 0 белых кадров); внешних запросов 0 | **done** (2026-11-06); P2-follow-ups: живой WebView2, IPC/SQLite-синхронизация темы (D-20) |
| T-0104 SQLite | 1.0 | БД, миграции, 2 репозитория; фикстура `v0_empty.db`; AC-1 измерен (медиана 10.28 мс), AC-4 проверен вручную; 123 Rust-теста; CI run 42152167354 зелёный | **done** (2026-11-06); 4 репозитория отложены (D-13) |

T-0101, T-0103 и T-0104 закрыты. Статус `done` подтверждён локальными
прогонами, ручными AC и зелёным CI; у T-0102 осталась только P2-часть.

### Гейт G1

В G1 **7 критериев**; оконный критерий перенесён в G2 (D-16). Все 7
подтверждены 2026-11-06, подробная таблица — в `STATUS.md`, раздел «Gate G1»:

- идемпотентность `ipc:gen`, идемпотентность миграций, тесты и покрытие
  93.68 % в CI, зелёный CI, страница с токенами и шрифтом;
- переключение темы без вспышки — запись `theme-switch-240fps.mp4`;
- README «за 3 команды» — три команды из корня репозитория.

---

## 2. Что физически есть

```
vela-vijets/                     корень монорепозитория, git main
├── .github/workflows/ci.yml     CI: rust, coverage, ui, ipc-contracts, desktop (пути → vela-app)
├── AGENTS.md                    инструкции агентам и аудит противоречий
├── STATUS.md, SESSION.md        ссылки на канонические копии в vela-app/docs
├── vela-plan/                   189 Markdown-документов, manifest.json
├── vela-demo/                   прототип и 4 скриншота
└── vela-app/
    ├── apps/desktop/            Tauri 2.12.5: app, tray, surfaces, cli, commands;
    │                            на не-Windows — headless-заглушка
    ├── crates/
    │   ├── vela-core/           db, element, error, lib, migrations, paths, repo
    │   │   └── migrations/      0001_init.sql — 8 таблиц; fixtures/v0_empty.db
    │   ├── vela-ipc/            контракты, contract.json, кодоген, SurfaceKind/Status
    │   ├── vela-platform/       Win32-обёртки (win.rs), health
    │   └── vela-cron/           заглушка
    ├── ui/                      Vite + React + TS
    │   ├── src/shared/api/      10 сгенерированных + 10 рукописных файлов
    │   ├── src/features/surfaces/ SurfacesPanel — первая живая IPC-панель
    │   ├── src/shared/theme/    theme.ts, useTheme.ts (+ тесты)
    │   ├── src/shared/ui/       Glass, Button, Switch, Slider, Field, Kbd, Divider, Skeleton
    │   ├── src/styles/          tokens, base, typography, glass, ui-kit
    │   └── src/assets/fonts/    Inter Tight woff2 (cyrillic + latin) и OFL.txt
    └── scripts/arch-check.sh    архитектурные запреты по манифестам
```

Tauri и окно UI уже есть; холста, виджетов, сцены, захвата и `bench/`
по-прежнему нет. Витрина `/_kit` существует только в dev.

---

## 3. Проверки: что подтверждено, а что нет

### Прогнано 2026-11-15 (T-0201, Windows-GNU)

| Проверка | Результат |
|---|---|
| `cargo test --workspace --all-features` | ✓ 148 тестов |
| `cargo fmt --all --check` | ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✓ |
| `pnpm typecheck` / `lint` / `format:check` | ✓ |
| `pnpm test` | ✓ 76/76 в 12 файлах |
| `pnpm ipc:check` | ✓ 6 тестов, «типы актуальны» |
| `bash scripts/arch-check.sh` | ✓ |
| Release: старт | ✓ UI скрыто за 84–169 мс, трей за 656–692 мс |
| Release: второй экземпляр | ✓ 44.7–45.4 мс, exit 0 |
| Release: трей | ✓ медиана переключения 21.71 мс, максимум 35.94 мс |
| Release: «Выход» | ✓ exit 0, 0 процессов |
| Release: Alt+F4 | ✓ окно скрылось, процесс жив, ex-style не изменился |
| T-0102 AC-2 | ✓ переименование поля уронило `tsc` 6 ошибками; после `ipc:gen` зелено |
| CI PR #2 | ✓ run 43697165125: `rust`, `coverage`, `ui`, `ipc-contracts`, `desktop` — success |

### Прогнано 2026-11-05 (UI)

| Проверка | Результат |
|---|---|
| `pnpm typecheck` | ✓ |
| `pnpm lint` | ✓ 0 предупреждений |
| `pnpm test` | ✓ 70/70 в 11 файлах |
| `pnpm format:check` | ✓ |
| `pnpm build` | ✓ 38 модулей, `KitPage` вырезана из production |
| `pnpm dev` | ✓ Vite на 5173; `/` и `/_kit` отвечали 200; скриншоты сняты в headless Edge |

### Прогнано 2026-11-06 (UI, CDP и запись экрана)

| Проверка | Результат |
|---|---|
| AC-2 T-0103: запись 240 fps | ✓ `theme-switch-240fps.mp4` — 1920×1080, 240 fps CFR, 1200 кадров, 5.000 с; максимум среднего RGB 243/241/245, минимум 15/13/17, кадров с R,G,B ≥ 252 — 0 |
| TC-0103-03: 20 переключений | ✓ тема чередуется, `<style>`/DOM/stylesheet не растут, ошибок 0 |
| ST-0103-01 AC-2: сеть | ✓ 34 запроса, все к `localhost:5173` + один `data:`-URI; внешних хостов 0; Inter Tight `status: loaded` |
| branch protection `main` | ✓ API: `strict=true`, контексты `rust`, `coverage`, `ui`, `ipc-contracts`, `enforce_admins=true`; force-push и удаление запрещены |

### Прогнано 2026-11-06 (CI, GitHub Actions)

| Проверка | Результат |
|---|---|
| run [42152167354](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42152167354) | ✓ success, 4 job'а |
| run [42154029664](https://github.com/romanbigaltay-bot/vela-widgets/actions/runs/42154029664) на `89147eb` | ✓ success, 4 job'а |
| `rust`, `coverage`, `ui`, `ipc-contracts` | ✓ |
| покрытие строк | 93.68 % (порог 60 %) |

### Прогнано 2026-11-06 (Rust, Windows-GNU)

| Проверка | Результат |
|---|---|
| `cargo fmt --all --check` | ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✓ без замечаний |
| `cargo test --workspace` | ✓ 118 тестов |
| `cargo test --workspace --all-features` | ✓ 123 теста |
| `bash scripts/arch-check.sh` | ✓ |
| `cargo check -p vela-core --example bench_startup` | ✓ |

### Прогнано 2026-11-03 (Rust, Windows-GNU)

| Проверка | Результат |
|---|---|
| `cargo fmt --all --check` | ✓ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✓ без замечаний |
| `cargo test --workspace` | ✓ 117 тестов |
| `cargo test --workspace --all-features` | ✓ 122 теста |
| `pnpm ipc:check` | ✓ 5 тестов, типы актуальны |
| `bash scripts/arch-check.sh` | ✓ |
| `cargo run -p vela` | ✓ exit 0, `state.db` создан |

### Подтверждено чтением

| Проверка | Результат |
|---|---|
| Git | `main` — `4b560e0`; ветка `feat/t-0201-tauri-app` ([PR #2](https://github.com/romanbigaltay-bot/vela-widgets/pull/2)), коммиты `256dfd5`, `e6fbb62`, `906f06a`; remote `romanbigaltay-bot/vela-widgets` (public) |
| Схема БД | 8 таблиц в `0001_init.sql` + bootstrap `schema_migrations` = 9 |
| Контракты | 10 generated, 10 handwritten; `contract.json` читают Rust и JS; `InvokeMap` содержит `window.surfaces_status` |
| UI-тесты | 76 тестов в 12 файлах; T-0201 добавила `SurfacesPanel` |
| Rust-тесты | 148 с `--all-features`; 13 в `vela-platform`, 12 в `vela` |
| Токены | 9 spacing 6–64, шкала 12–56, dark/light, `--accent-text` |
| CI | run 42152167354 и 42154029664 — success; jobs `rust`, `coverage`, `ui`, `ipc-contracts`; PR #2 (run 43697165125) — все 5 job'ов success, включая новый `desktop` (windows-latest) |
| Защита `main` | branch protection: `strict=true`, обязательные проверки `rust`, `coverage`, `ui`, `ipc-contracts`, `enforce_admins=true` |
| Покрытие | 93.68 % строк в CI (порог 60 %) |
| Документы | 189 `.md` в `vela-plan` |
| T-0104 AC-1 | `examples/bench_startup.rs`: 20 прогонов, медиана 10.28 мс при пороге 50 мс |
| T-0104 AC-4 | мусорный `state.db` → `state.db.corrupt-*` (SHA-256 совпал), новая БД, exit 0 |

### Не подтверждено

- Покрытие локально не измеряется: `cargo llvm-cov` не работает на Windows-GNU
  (`profiler_builtins`); в CI — 93.68 % строк.
- Живой WebView2, холодный старт и реальное событие смены системной темы
  (ST-0103-02) станут проверяемыми в P2; IPC/SQLite-синхронизация темы — D-20.
- Windows-джоба `desktop` впервые прошла в PR #2 (run 43697165125): все
  5 job'ов success; локально clippy/build/test тоже зелёные.
- Сборка под MSVC не проверялась; локально стоит GNU-toolchain.
- `--safe-mode` пока только разбирается; проверка на сломанном WorkerW —
  T-0202 (D-25).

---

## 4. Что сделано 2026-11-02…15

- Инициализирован git в корне `vela-vijets`, создан `.gitignore`, первый
  коммит `57fb5a0`; затем `a217cb5` (CI/инструменты) и `d526809` (документы).
- CI перенесён в корневой `.github/workflows/ci.yml`; все команды и кэш
  указывают на `vela-app`, старый workflow удалён.
- `ipc-gen.mjs` читает `contract.json` вместо захардкоженного списка.
- ESLint игнорирует сгенерированные контракты по списку из `contract.json`.
- Удалён несуществующий скрипт `pnpm size`.
- G1 сокращён до 7 критериев; оконный критерий перенесён в G2.
- ADR-0005 приведён к фактической схеме; `03-architecture.md` и ST-0104-01 —
  к потоку-владельцу без tokio.
- PLAN.md: исправлены счёт документов (189), граф зависимостей и политика
  blur mirror.
- Статусы P1 приведены к факту: 0 закрытых, 3 в работе, 1 не начата.
- Корневые `STATUS.md`/`SESSION.md` заменены ссылками на канонические копии.
- Исправлены doc-комментарии `repo.rs`, `element.rs`, `commands.ts` и
  `vela-core/Cargo.toml`.
- Установлены Rust 1.99.1 GNU, MinGW GCC 16.2.0, pnpm 9.15.9.
- Прогнаны fmt, clippy, 122 Rust-теста, 34 UI-теста, `ipc:check`, arch-check,
  `pnpm build`; `cargo run -p vela` создал БД и вышел с кодом 0.
- В CI добавлена job `coverage` с порогом 60 % (D-17).
- `export_bindings` больше не требует `TS_RS_EXPORT_DIR` для контентных
  тестов: они пишут во временный каталог.

### 2026-11-05 — T-0103 (код; задача не закрыта)

- `tokens.css`: dark/light, 9 spacing 6–64, крупная шкала, радиусы, тени, стекло,
  движение, `prefers-reduced-motion`; `--accent-text` для AA-контраста.
- `typography.css`: 7 классов от `.t-micro` до `.t-hero`; после визуальной
  проверки шкала увеличена до 12–56 px, caption — 14 px.
- `ui-kit.css`: кнопки, поля, switch, slider и отступы крупнее; spacing
  6–64, радиусы 6/12/20. В светлой теме текст на акценте белый, accent
  затемнён до `#4a68e0` ради AA; в тёмной остаётся чёрный. Витрина
  прокручивается внутри `.kit` (`100dvh` + `overflow-y: auto`), иерархия
  отступов — 64/32/24 px.
- `glass.css` и `ui-kit.css`: `.glass` с `::before`/`::after`, фолбэк через
  `@supports`, компоненты и dev-витрина.
- `ThemeService`: inline-скрипт до бандла, `matchMedia`, `localStorage`,
  `storage`-событие, `setThemeFromHost` под будущее событие P2.
- `Glass` с dev-предупреждением о вложенности; `Button`, `IconButton`,
  `Switch`, `Slider`, `Field`, `Kbd`, `Divider`, `Skeleton`.
- Витрина `/_kit` грузится лениво и только в dev; production-бандл уменьшился
  с 49 до 38 модулей.
- `vitest.setup.ts` получил in-memory `localStorage`: глобальный Node 25 без
  `--localstorage-file` не имеет `clear()` и ронял тесты темы.
- Тесты: `tokens` 6, `theme` 10, `theme-boot` 6, `Glass` 3, `Button` 4,
  `Switch` 4, `KitPage` 3. Итого 70/70.

### 2026-11-05 — T-0104 (код и AC; задача не закрыта)

- Добавлена фикстура `migrations/fixtures/v0_empty.db` (0 байт) и тест
  `tc_0104_02_migrates_v0_empty_fixture`: пустой файл открывается как
  `OpenOutcome::Opened` и мигрирует v0 → v1 без бэкапа.
- AC-1 измерен `examples/bench_startup.rs` (release, 20 прогонов на временной
  БД): первый старт `Db::open + migrate` — лучший 8.68 мс, медиана 10.28 мс,
  средний 10.72 мс при пороге 50 мс; повторный `migrate` — 0.08–0.10 мс.
- AC-4 проверен вручную release-бинарем: мусорный `state.db` переименован в
  `state.db.corrupt-*`, SHA-256 копии совпал; создана новая БД (9 таблиц,
  `schema_version=1`), процесс завершился с кодом 0.
- 2026-11-06: fmt, clippy, arch-check и 123 теста (`--all-features`) зелёные;
  без фич — 118.
- На 2026-11-05 задача оставалась `in-progress` только из-за T-0101: не было
  удалённого репозитория и прогона CI.

### 2026-11-06 — T-0104 закрыта, первый CI

- Remote `github.com/romanbigaltay-bot/vela-widgets` создан, `main` запушен.
- CI run 42152167354: `rust`, `coverage`, `ui`, `ipc-contracts` — success;
  покрытие строк 93.68 % при пороге 60 % (D-17 закрыт).
- T-0104 переведена в `done`, `closed_at: 2026-11-06`; подзадачи
  ST-0104-01…03 тоже `done`.
- T-0102: AC-2 (CI) закрыт; остался ручной AC-4.
- T-0101: remote и CI готовы; на тот момент оставались AC-1, AC-2 и AC-6
  (закрыты позже в этой же сессии).

### 2026-11-06 — T-0101 и T-0103 закрыты, G1 пройден

- Репозиторий переведён в public; через API включена branch protection `main`:
  обязательные `rust`, `coverage`, `ui`, `ipc-contracts`, `strict=true`,
  `enforce_admins=true`, force-push и удаление запрещены.
- CI run 42154029664 на `89147eb`: все четыре job'а success.
- Установлен ffmpeg 9.0.2 (winget); запись темы сделана в Chrome kiosk через
  CDP, `libx264` (NVENC не подошёл: драйвер 13.0 при требуемом 13.1).
- `visual/theme-switch-240fps.mp4`: 1920×1080, 240 fps CFR, 1200 кадров,
  5.000 с; покадровый анализ — 0 кадров белее светлой темы.
- `visual/theme-switch-240fps.md`: протокол записи и ограничение 60 Гц
  монитора (240 fps получены CFR-дублированием).
- ST-0103-01 AC-2: CDP Network — 34 запроса, внешних хостов 0, Inter Tight
  загружен.
- T-0101, T-0103, ST-0103-01 переведены в `done`; `manifest.json` обновлён,
  `closed_at: 2026-11-06`. T-0102 остаётся `in-progress` (P2-часть).
- `vela-app/README.md`: быстрый старт сведён к трём командам из корня.
- G1 пройден 7/7.

### 2026-11-15 — T-0201 и T-0102 закрыты

- Подключён Tauri 2.12.5 (`tauri-build` 2.7.4, `single-instance` 2.4.x);
  зависимости Windows-only, на других ОС — headless-заглушка.
- `apps/desktop/src/app.rs`: `RunEvent::Ready`, трей, Alt+F4 через
  `prevent_close`, `ExitRequested` отменяется без кода, выход через трей.
- `apps/desktop/src/surfaces.rs`: `SurfaceManager`, создание UI 800×600
  скрытой, `ensure` идемпотентен, `Drop`/`destroy_all`, `status()`.
- `vela-platform::win`: `ExStyle` (`APPWINDOW`, `without`),
  `remove_ex_style`, `set_window_visible`, `is_window_visible`,
  `set_foreground`; оконные тесты сериализованы `WINDOW_TEST_LOCK`.
- Показ/скрытие — через `ShowWindow`, а не `window.show()`: tao переписывает
  `GWL_EXSTYLE` и возвращает `WS_EX_APPWINDOW`. Стили повторно
  применяются до и после `SetForegroundWindow`; фактический ex-style
  `0x08000190`.
- Событие переименовано в `window:surfaces_changed` — Tauri 2 не принимает
  точки в именах.
- UI: `SurfacesPanel` вызывает `window.surfaces_status`, показывает снимок и
  обновляется по событию; `App.test.tsx` и `client.test.ts` обновлены.
- `ipc-gen.mjs` теперь действительно пишет файлы, а `--check` не мутирует
  дерево; добавлен Rust-тест `export_to_configured_dir`.
- `build.rs`: `embed-resource` линкует манифест Common-Controls и в тестовые
  бинарники; иначе тестовый exe падал с `STATUS_ENTRYPOINT_NOT_FOUND`.
- CI: добавлена Windows-джоба `desktop` (clippy, build, test).
- Замеры release: UI скрыто за 84–169 мс, трей за 656–692 мс, второй
  экземпляр 44.7–45.4 мс (exit 0), переключение трея медиана 21.71 мс,
  «Выход» exit 0.
- T-0102 закрыта: `InvokeMap` содержит первую команду; переименование поля
  в `SurfaceStatus.ts` уронило `tsc` 6 ошибками, после `ipc:gen` зелено.
- 148 Rust-тестов (`--all-features`), 76 UI-тестов; fmt, clippy, typecheck,
  lint, format, `ipc:check`, arch-check — зелёные.
- Ветка `feat/t-0201-tauri-app`, PR #2; CI run 43697165125 — все 5 job'ов
  success, включая `desktop`; после merge — T-0202.

---

## 5. Примечания: что важно помнить

### 5.1 Ловушки, на которые уже наступали

1. **`languageOptions.globals` в ESLint заменяет набор, а не дополняет.**
   Добавление `globals.node` к UI-файлам стирает `window`/`document`.
2. **`.editorconfig` управляет отступами для Prettier.** Не покрыл `.mjs` —
   Prettier и ESLint разошлись.
3. **Prettier переформатирует сгенерированные файлы.** Нужен
   `.prettierignore` на `src/shared/api/*.ts`.
4. **`unwrap_used = deny` ломает тесты.** Исключение — `cfg_attr(test,
   allow(…))` на уровне крейта.
5. **`rusqlite::row.get::<_, String>` на `NULL` падает.** Нужен
   `Option<String>`.
6. **`std::env::set_var` в тестах недетерминирован** — env инжектируется
   параметром (`paths::data_dir_from`).
7. **Замыкание в `Db::call` требует `'static`** — данные переезжают `move`.
8. **`shutdown(&mut self)`** — в тестах нужен `let mut db`.
9. **`TS_RS_EXPORT_DIR` с относительным путём** резолвится от каталога теста;
   резолвить от корня workspace.
10. **Clippy `assertions_on_constants`** — нужен `const _: () = assert!(…)`.

### 5.2 Отклонения, влияющие на следующую работу

| ID | Отклонение | Влияние |
|---|---|---|
| D-5 | 5 токенов вместо полного набора | **закрыто 2026-11-05**: токены, темы, ui-kit на месте; T-0103 закрыта 2026-11-06 |
| D-13 | 4 репозитория не созданы | создавать в T-0305, ST-0404-01, ST-0704-02, T-0504 |
| D-14 | Неизвестный `kind` читается как `Widget` | добавить `ElementKind::Unknown` в ST-0302-01 |
| D-16 | Окно 800×600 было в G1 | **решено**: критерий перенесён в G2 |
| D-17 | Покрытие не измерено | **закрыто 2026-11-06**: 93.68 % строк в CI |
| D-18 | `zod`-валидация отложена | транспорт появился в T-0201, но валидация в объём не входила; включать отдельной задачей |
| D-20 | Тема в P1 хранится в `localStorage` | в P1 не было Tauri/команды; SQLite и `app.theme_changed` — ST-0103-02, inline-скрипт остаётся |
| D-21 | Меню трея сокращено до «Показать виджеты» и «Выход» | расширять по мере появления T-0202…T-0205; неработающий пункт хуже отсутствующего |
| D-22 | Тост «Vela уже запущена» не показан | вернуться вместе с настройками/уведомлениями; сейчас второй экземпляр просто показывает UI |
| D-23 | Событие — `window:surfaces_changed`, а не `window.surfaces_changed` | Tauri 2 не принимает точки в именах; план и UI обновлены |
| D-24 | UI создаётся с `transparent: false`, `parent_to_workerw` — `Unsupported` | прозрачность — T-0203, WorkerW — T-0202; не маскировать |
| D-25 | `--settings`/`--no-scene`/`--safe-mode` только разбираются | настройки — T-0204, сцена/WorkerW — T-0202; проверка `--safe-mode` перенесена |
| D-26 | Windows-джоба `desktop` в CI, покрытие считается на Linux | Win32/Tauri не компилируется на Linux; `desktop` проверяет clippy/build/test; первый зелёный прогон — PR #2 (run 43697165125) |
| D-27 | `cargo run -p vela` — dev-режим и грузит `devUrl` | автономный бинарь — `--features tauri/custom-protocol` после `pnpm build`; README описывает оба режима |

---

## 6. С чего начинать следующую сессию

### Приоритет 1: T-0202 — поверхность A (WorkerW)

T-0201 закрыта: приложение живёт в трее, UI-поверхность создаётся скрытой,
второй экземпляр активирует первый, Alt+F4 скрывает. Следующий шаг — WorkerW:
привязка сцены под иконки, геометрия/DPI и восстановление после Explorer.
Перед началом открыть `vela-plan/02-shell/T-0202-workerw-scene/`, а не работать
по памяти.

### Приоритет 2: T-0203 — поверхность B (UI)

Прозрачность, hit-regions и режимы ввода. Можно вести параллельно с T-0202:
обе задачи используют `SurfaceManager` из T-0201, но трогают разные окна.

### Приоритет 3: T-0103 P2-follow-ups

- Живой WebView2: шрифт, фокус, стекло.
- ST-0103-02: холодный старт, реальное событие смены системной темы,
  IPC/SQLite-синхронизация (`app.theme_changed`, D-20).

### Что НЕ делать

- Не возвращать `localStorage`-тему как постоянное решение: в P2 источник —
  SQLite, inline-скрипт анти-вспышки остаётся.
- Не добавлять `ProfilesRepo`/`NotesRepo` «на будущее» (D-13).
- Не тянуть tokio в `vela-core` (D-7).
- Не добавлять `bench/` раньше T-0802 (P8).
- Не считать статический подсчёт тестов прогоном.

---

## 7. Как быстро проверить состояние

```bash
cd vela-vijets
git log --oneline -3
git status --short
git remote -v

cd vela-app
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --all-features
bash scripts/arch-check.sh

cd ui
pnpm install --frozen-lockfile
pnpm typecheck && pnpm lint && pnpm test && pnpm format:check && pnpm ipc:check
```

На 2026-11-15 это подтверждено: 148 тестов с `--all-features`, 76 UI-тестов,
`ipc:check` без диффа, `arch-check` зелёный. CI PR #2 (run 43697165125) —
все 5 job'ов success; покрытие по-прежнему считается на Linux и локально не
измеряется. Если числа расходятся с STATUS — прав статус, а не документ.

---

## 8. Оценка темпа

По плану P1 — 4 дня. Все четыре задачи реализованы и **закрыты**
(T-0101, T-0103, T-0104 — 2026-11-06; T-0102 — 2026-11-15 после первой
реальной команды). G1 пройден 7/7.

P2 «Оболочка»: по плану 8 дней на 5 задач. Закрыта T-0201 (Tauri, трей,
single-instance, `SurfaceManager`, скрытая UI-поверхность 800×600) —
фактически 2 рабочих дня с замерами и документацией. Следующая — T-0202
(WorkerW), затем T-0203 (UI-поверхность); их можно вести параллельно.
