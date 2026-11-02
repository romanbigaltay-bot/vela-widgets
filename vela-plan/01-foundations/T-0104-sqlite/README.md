---
id: T-0104
stage: P1
title: SQLite-схема, миграции, репозитории
status: ready
est: 1.0d
owner: rust
depends: [T-0101]
gate: G1
---

# T-0104 — SQLite-схема, миграции, репозитории

## 1. Цель

Единственный источник истины для раскладки, настроек, заметок и истории.
С первого дня — с миграциями, иначе вторая версия ломает данные первых
пользователей.

## 2. Схема v1

```sql
-- settings: единый key-value с типизированным JSON
CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at INTEGER NOT NULL);

-- monitors: геометрия на момент запуска, для восстановления раскладки
CREATE TABLE monitors (
  id TEXT PRIMARY KEY, x INTEGER, y INTEGER, w INTEGER, h INTEGER,
  scale REAL, primary_ BOOLEAN, last_seen INTEGER
);

-- profiles: набор сцены + раскладки
CREATE TABLE profiles (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, scene_json TEXT NOT NULL,
  is_active BOOLEAN DEFAULT 0, sort INTEGER DEFAULT 0
);

-- elements: всё, что лежит на холсте
CREATE TABLE elements (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,            -- 'widget'|'pin'|'note'|'ink'|'group'
  widget_id TEXT,                -- 'core.clock' для kind='widget'
  monitor_id TEXT NOT NULL,
  profile_id TEXT,               -- NULL = во всех профилях
  x REAL NOT NULL, y REAL NOT NULL,
  w REAL NOT NULL, h REAL NOT NULL,
  z INTEGER NOT NULL DEFAULT 0,
  rotation REAL DEFAULT 0,
  settings_json TEXT NOT NULL DEFAULT '{}',
  state_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL
);
CREATE INDEX idx_elements_monitor ON elements(monitor_id, profile_id);

-- notes: отдельно от elements (много текста, поиск)
CREATE TABLE notes (element_id TEXT PRIMARY KEY REFERENCES elements(id) ON DELETE CASCADE,
  title TEXT, body TEXT, checklist_json TEXT);

-- cron_jobs
CREATE TABLE cron_jobs (id TEXT PRIMARY KEY, expr TEXT NOT NULL, action_json TEXT NOT NULL,
  enabled BOOLEAN DEFAULT 1, next_fire_at INTEGER, last_fire_at INTEGER, catch_up BOOLEAN DEFAULT 1);

-- capture_history (метаданные, не сами файлы)
CREATE TABLE capture_history (id TEXT PRIMARY KEY, path TEXT, rect_json TEXT,
  created_at INTEGER, ocr_text TEXT);

-- schema_migrations
CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);
```

## 3. Решения

- **WAL** включён (`PRAGMA journal_mode=WAL`), `synchronous=NORMAL`,
  `foreign_keys=ON`.
- Миграции — простые SQL-файлы `migrations/0001_init.sql`, применяются по порядку
  в транзакции, версия в `schema_migrations`. Откат не поддерживается (данные
  пользователя), только forward-миграции + бэкап перед применением.
- Репозитории: трейты в `vela-core::repo` (`Elements`, `Settings`, `Profiles`,
  `Notes`), реализация на `rusqlite`, доступ через `Arc<Mutex<Connection>>`
  в отдельном потоке (или пул `r2d2_sqlite` для параллельных чтений).
- Запись раскладки — одна транзакция на `pointerup`, дебаунс 250 мс; при
  закрытии приложения — синхронный flush.

## 4. Состав подзадач

| ID | Подзадача | Дней |
|---|---|---|
| ST-0104-01 | Слой БД, подключение, PRAGMA, открытие/создание | 0.3 |
| ST-0104-02 | Миграции и начальная схема | 0.3 |
| ST-0104-03 | Репозитории и тесты на временной БД | 0.4 |

## 5. Критерии приёмки

- **AC-1** Первый запуск создаёт БД за ≤ 50 мс, повторный — не трогает схему.
- **AC-2** Миграция с нуля идемпотентна: два запуска подряд дают одинаковый
  `schema_migrations`.
- **AC-3** Удаление элемента каскадно удаляет его заметку.
- **AC-4** БД повреждена (подменён файл) → приложение стартует с пустой БД,
  старое переименовано в `state.db.corrupt-<ts>`, пользователь уведомлён.
- **AC-5** Тесты используют `tempfile`, не трогают реальную БД.

## 6. Тест-план

| ID | Проверка |
|---|---|
| TC-0104-01 | Миграция с нуля на пустом файле |
| TC-0104-02 | Миграция на БД версии N-1 (фикстура) |
| TC-0104-03 | Каскадное удаление notes → elements |
| TC-0104-04 | Имитация порчи файла (записать мусор) → восстановление |

## 7. Риски

- Блокировки WAL при двух процессах (приложение + отладка) → единственный
  процесс-владелец, `busy_timeout = 3000`.
- Схема будет меняться в P3/P7 → правило: миграции только добавляющие,
  `elements.state_json` для нестабильных данных.
