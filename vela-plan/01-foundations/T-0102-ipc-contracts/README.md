---
id: T-0102
stage: P1
title: Контракты IPC и генерация TS-типов
status: done
est: 1.0d
owner: rust
depends: [T-0101]
gate: G1
---

# T-0102 — Контракты IPC и генерация TS-типов

## 0. Фактическое состояние (2026-11-15)

T-0102 закрыта 2026-11-15: P2-часть подтверждена первой реальной командой.

- `ts-rs` пишет **по файлу на тип** (`Envelope.ts`, `ErrorCode.ts`, …), а не
  один `types.gen.ts`; `index.ts` рукописный и реэкспортирует типы;
- `InvokeMap` в `commands.ts` содержит `window.surfaces_status`
  (`req: undefined`, `res: SurfaceStatus[]`). Индексной сигнатуры
  `[command: string]` в `CommandMap` нет, поэтому опечатка в имени команды —
  ошибка `tsc`, а не пустой экран;
- dev-валидация через `zod` по-прежнему отложена (D-18): транспорт есть, но
  валидация не входила в объём T-0201;
- `ipc-gen.mjs` читает `contract.json`, экспортирует во временный каталог,
  сравнивает с рабочим и не мутирует дерево в режиме `--check`;
- AC-2 подтверждён в CI 2026-11-06 (job `ipc-contracts`, run 42152167354);
- AC-4 подтверждён 2026-11-06: на живом Vite-сервере через CDP вызван
  `call('probe.echo', { ping: 1 })`; в dev-консоли появился
  `ipc probe.echo 0ms ok` (формат `{cmd} {ms}ms ok`);
- 2026-11-15 AC-2 проверен на реальной команде: временное переименование поля
  `kind` → `kindRenamed` в сгенерированном `SurfaceStatus.ts` уронило `tsc`
  в 4 файлах (6 ошибок); после `pnpm ipc:gen` typecheck снова зелёный.

Статус — `done` (2026-11-15).

## 1. Цель

Один источник истины для всех сообщений между Rust и UI. Ни одно UI-обращение
к системе не должно проходить мимо типизированной команды.

## 2. Решение

- Крейт `vela-ipc`: чистые `serde`-структуры, без `tauri` в зависимостях
  (иначе типы нельзя использовать в тестах домена).
- Генерация TS через `ts-rs` в `ui/src/shared/api/` — по файлу на тип
  (`Envelope.ts`, `VelaError.ts`, …); рукописный `index.ts` реэкспортирует их.
  Файлы коммитятся, расхождение ловится `pnpm ipc:check`.
- Общая обёртка `invoke`: логирует имя команды, время выполнения и ошибку.
  Dev-валидация payload через `zod` отложена до P2, когда появится транспорт
  и реальные команды (D-18).
- События (`emit`/`listen`) — тоже типизированы, единый реестр `EventMap`.

## 3. Группы команд

| Группа | Примеры | Этап |
|---|---|---|
| `layout.*` | `layout.get`, `layout.apply_patch`, `layout.reset` | P3 |
| `window.*` | `window.set_mode`, `window.surfaces_status` | P2 |
| `profile.*` | `profile.list`, `profile.activate` | P3 |
| `widget.*` | `widget.manifest_list`, `widget.settings_update` | P4 |
| `scene.*` | `scene.set_source`, `scene.pause` | P6 |
| `capture.*` | `capture.request`, `capture.annotate_save` | P5 |
| `cron.*` | `cron.list`, `cron.upsert`, `cron.run_now` | P7 |
| `system.*` | `system.metrics`, `system.power_state` | P7 |
| `app.*` | `app.settings`, `app.autostart_set`, `app.quit` | P2 |

## 4. Форма ответа

```rust
#[derive(Serialize, TS)]
#[ts(export)]
pub struct Envelope<T> { pub ok: bool, pub data: Option<T>, pub error: Option<VelaError> }

#[derive(Serialize, TS)]
pub struct VelaError { pub code: ErrorCode, pub message: String, pub retryable: bool }
```

Коды ошибок фиксированы перечислением (`PermissionDenied`, `NotFound`,
`PlatformUnavailable`, `Busy`, `InvalidInput`, `Internal`). UI обрабатывает
`code`, а не текст.

## 5. Состав подзадач

| ID | Подзадача | Дней |
|---|---|---|
| ST-0102-01 | Крейт `vela-ipc`: типы, конверт, ошибки | 0.3 |
| ST-0102-02 | Кодоген TS + CI-проверка | 0.3 |
| ST-0102-03 | Обёртка `invoke` в UI + реестр событий | 0.4 |

## 6. Критерии приёмки

- **AC-1** `pnpm ipc:gen` идемпотентна: повторный запуск не меняет файл.
- **AC-2** Переименование поля в Rust ломает `tsc` (проверяется тестом в CI).
- **AC-3** Каждая ошибка имеет код из перечисления; произвольные строки запрещены.
- **AC-4** Обёртка `invoke` логирует длительность; видна в dev-консоли.
- **AC-5** Тест: вызов команды без права возвращает `PermissionDenied`, не паникует.

## 7. Тест-план

| ID | Проверка |
|---|---|
| TC-0102-01 | Юнит: сериализация `Envelope` с ошибкой и без |
| TC-0102-02 | Интеграция: `ipc:gen` в CI без диффа |
| TC-0102-03 | Ручная: сломать тип → увидеть ошибку компиляции TS |

## 8. Риски

- `ts-rs` не умеет некоторые типы (например, `HashMap` с нестроковыми ключами) →
  правило: ключи только `String`.
- Двойная генерация (для A и B поверхностей) даст два файла → один общий файл,
  импортируется обеими точками входа.

## 9. Доказательства (2026-11-06)

- AC-1 — `pnpm ipc:check`: 5 контрактных тестов, «типы актуальны».
- AC-2 — CI run 42152167354, job `ipc-contracts`: success.
- AC-3 — `ErrorCode` — закрытый enum; `wire_format_is_snake_case` и
  `every_error_code_exists_in_generated_binding`.
- AC-4 — CDP на живом Vite: в dev-консоли `ipc probe.echo 0ms ok`.
- AC-5 — `client.test.ts` «сохраняет код, сообщение и флаг retryable»:
  `PermissionDenied` приходит как `VelaFailure` с `code`, без паники.

## 10. Доказательства P2-части (2026-11-15)

- AC-2: в `ui/src/shared/api/SurfaceStatus.ts` временно переименовано поле
  `kind` в `kindRenamed` (эквивалент переименования в Rust без регенерации);
  `pnpm typecheck` упал с 6 ошибками в `App.test.tsx`,
  `SurfacesPanel.test.tsx`, `SurfacesPanel.tsx` и `client.test.ts`; после
  `pnpm ipc:gen` и `pnpm ipc:check` typecheck снова зелёный.
- AC-3: `InvokeMap['window.surfaces_status']` имеет `req: undefined` и
  `res: SurfaceStatus[]`; обработчик `commands::window_surfaces_status`
  зарегистрирован в Tauri, UI-панель `SurfacesPanel` вызывает команду через
  `call()` и получает типизированный ответ.
- AC-1: `pnpm ipc:check` — «типы актуальны», 6 контрактных тестов.
- AC-4: `SurfacesPanel` в живом приложении показывает снимок поверхности,
  то есть цепочка `React → call() → Tauri → Rust → Envelope → React`
  пройдена на настоящем окне.
