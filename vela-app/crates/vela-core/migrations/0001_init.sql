-- Схема v1 Vela.
--
-- Правила для файлов в этом каталоге (нарушение ломает откат):
--
--  1. Без `IF NOT EXISTS`. Повторное применение обязано падать с ошибкой, а не
--     проходить молча: идемпотентность обеспечивает таблица `schema_migrations`,
--     а не робость DDL. Молчаливый `IF NOT EXISTS` скрывает расхождение между
--     схемой на диске и тем, что думает код.
--  2. Без `BEGIN` / `COMMIT`. Транзакцию открывает мигратор; вложенная
--     транзакция сломала бы откат при падении посреди файла.
--  3. Один файл — одна версия, номер в имени. Порядок применения — по номеру,
--     не по алфавиту.
--
-- Временные метки везде i64 миллисекунды UTC (см. ST-0104-03).

-- Настройки: единый key-value, значение — JSON. Отдельные колонки под каждую
-- настройку не заводим: набор меняется каждый этап, а миграция на каждую
-- новую галочку — это цена без пользы.
CREATE TABLE settings (
  key        TEXT PRIMARY KEY,
  value      TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

-- Мониторы: геометрия на момент запуска. Нужна, чтобы восстановить раскладку
-- после переподключения экрана (ST-0302-04).
--
-- `primary_` с подчёркиванием, потому что `primary` — ключевое слово SQL.
CREATE TABLE monitors (
  id        TEXT PRIMARY KEY,
  x         INTEGER,
  y         INTEGER,
  w         INTEGER,
  h         INTEGER,
  scale     REAL,
  primary_  BOOLEAN,
  last_seen INTEGER
);

-- Профили: именованный набор сцены и видимости элементов (T-0305).
CREATE TABLE profiles (
  id         TEXT PRIMARY KEY,
  name       TEXT NOT NULL,
  scene_json TEXT NOT NULL,
  is_active  BOOLEAN NOT NULL DEFAULT 0,
  sort       INTEGER NOT NULL DEFAULT 0
);

-- Всё, что лежит на холсте: виджеты, закреплённые снимки, заметки, рисунки.
--
-- `kind` намеренно без CHECK-ограничения. Набор видов будет расти (P4, P5), а
-- SQLite не умеет менять CHECK без перестройки таблицы — то есть каждая новая
-- разновидность элемента стоила бы миграции с копированием всех данных.
-- Допустимые значения проверяет домен (`Element::validate`, ST-0302-01): там
-- ошибка дешёвая, здесь — дорогая.
--
-- `monitor_id` намеренно без внешнего ключа: таблица `monitors` переписывается
-- при каждом запуске и не является справочником. Удаление монитора не должно
-- удалять элементы — их переносят в зону `orphaned` (ST-0302-04).
--
-- `profile_id` наоборот со ссылкой и `ON DELETE SET NULL`: удаление профиля по
-- ST-0305-01 обязано оставить элементы видимыми везде, и это правило лучше
-- держать в схеме, чем в коде, который про него забудут.
CREATE TABLE elements (
  id            TEXT PRIMARY KEY,
  kind          TEXT NOT NULL,
  widget_id     TEXT,
  monitor_id    TEXT NOT NULL,
  profile_id    TEXT REFERENCES profiles(id) ON DELETE SET NULL,
  x             REAL NOT NULL,
  y             REAL NOT NULL,
  w             REAL NOT NULL,
  h             REAL NOT NULL,
  z             INTEGER NOT NULL DEFAULT 0,
  rotation      REAL NOT NULL DEFAULT 0,
  settings_json TEXT NOT NULL DEFAULT '{}',
  state_json    TEXT NOT NULL DEFAULT '{}',
  created_at    INTEGER NOT NULL,
  updated_at    INTEGER NOT NULL
);

-- Основной запрос раскладки — «все элементы монитора для профиля» (ST-0302-02),
-- поэтому индекс составной и в этом порядке.
CREATE INDEX idx_elements_monitor ON elements (monitor_id, profile_id);

-- Заметки: много текста и поиск, поэтому отдельно от `elements`.
-- Каскад обязателен: заметка без элемента — это мусор, который никто не удалит.
CREATE TABLE notes (
  element_id     TEXT PRIMARY KEY REFERENCES elements(id) ON DELETE CASCADE,
  title          TEXT,
  body           TEXT,
  checklist_json TEXT
);

-- Задачи планировщика (T-0704). `next_fire_at` вычисляется при сохранении и
-- после каждого срабатывания, чтобы поток планировщика не разбирал cron-строки
-- на каждом тике.
CREATE TABLE cron_jobs (
  id           TEXT PRIMARY KEY,
  expr         TEXT NOT NULL,
  action_json  TEXT NOT NULL,
  enabled      BOOLEAN NOT NULL DEFAULT 1,
  next_fire_at INTEGER,
  last_fire_at INTEGER,
  catch_up     BOOLEAN NOT NULL DEFAULT 1
);

-- История снимков: только метаданные, сами файлы лежат в captures/ (T-0504).
-- Отдельная таблица, а не строка в settings: её чистит политика хранения.
CREATE TABLE capture_history (
  id         TEXT PRIMARY KEY,
  path       TEXT,
  rect_json  TEXT,
  created_at INTEGER,
  ocr_text   TEXT
);

-- Журнал исполнений планировщика (ST-0704-02). Уникальность пары
-- (job_id, fire_at) — это идемпотентность: повторное срабатывание не создаст
-- вторую запись и не выполнит действие дважды после сна или перезапуска.
CREATE TABLE cron_runs (
  job_id   TEXT NOT NULL,
  fire_at  INTEGER NOT NULL,
  status   TEXT NOT NULL,
  error    TEXT,
  ms       INTEGER,
  PRIMARY KEY (job_id, fire_at)
);
