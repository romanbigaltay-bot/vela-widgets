---
id: ST-0104-01
parent: T-0104
title: Слой БД, подключение, PRAGMA
status: ready
est: 0.3d
owner: rust
---

# ST-0104-01 — Слой БД

## Цель

Открытие/создание БД в `%LOCALAPPDATA%\Vela\state.db` с корректными PRAGMA
и восстановлением после порчи.

## Реализация

1. `vela-core::db::open(path) -> Result<Db>`; каталог создаётся с проверкой прав.
2. PRAGMA: `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`,
   `busy_timeout=3000`, `temp_store=MEMORY`.
3. Проверка целостности: `PRAGMA quick_check`; при ошибке — переименовать файл
   в `.corrupt-<ts>`, создать новый, вернуть `DbRecovered { backup_path }`.
4. `Db` держит `Connection` в выделенном потоке; API — `spawn_blocking`-обёртки,
   чтобы не блокировать tokio-рантайм.
5. `Db::flush()` для синхронной записи при выходе.

## Критерии приёмки

- AC-1 Открытие на чистом каталоге создаёт файл ≤ 100 КБ.
- AC-2 Мусор в файле → `DbRecovered`, приложение не падает.
- AC-3 Параллельные чтения из двух потоков не дают `SQLITE_BUSY`.

## Тест-план

Unit на `tempfile`; отдельный тест на «мусорный» файл; стресс — 100 чтений
из 4 потоков.

## Риски

`spawn_blocking` + `Mutex<Connection>` могут сериализовать всё; при росте
нагрузки — пул соединений (`r2d2_sqlite`).
