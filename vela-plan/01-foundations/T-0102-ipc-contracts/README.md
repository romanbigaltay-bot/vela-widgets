---
id: T-0102
stage: P1
title: Контракты IPC и генерация TS-типов
status: ready
est: 1.0d
owner: rust
depends: [T-0101]
gate: G1
---

# T-0102 — Контракты IPC и генерация TS-типов

## 1. Цель

Один источник истины для всех сообщений между Rust и UI. Ни одно UI-обращение
к системе не должно проходить мимо типизированной команды.

## 2. Решение

- Крейт `vela-ipc`: чистые `serde`-структуры, без `tauri` в зависимостях
  (иначе типы нельзя использовать в тестах домена).
- Генерация TS через `ts-rs` в `ui/src/shared/api/types.gen.ts`; файл коммитится,
  дифф проверяется в CI (`pnpm ipc:gen && git diff --exit-code`).
- Общая обёртка `invoke`: логирует имя команды, время выполнения и ошибку;
  в dev-режиме — валидирует payload по схеме.
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
