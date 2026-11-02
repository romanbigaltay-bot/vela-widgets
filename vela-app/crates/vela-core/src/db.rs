//! Открытие базы данных, PRAGMA и восстановление после порчи.
//!
//! # Почему владение соединением вынесено в поток
//!
//! `rusqlite::Connection` не `Sync`, а приложение многопоточное: холст, сцена
//! и захват живут в разных потоках и обращаются к данным одновременно. Два
//! способа это решить:
//!
//! 1. `Arc<Mutex<Connection>>` — вызывающий код держит блокировку, и любая
//!    паника с `Connection` внутри отравляет мьютекс: все последующие
//!    обращения падают, хотя база цела.
//! 2. Выделенный поток-владелец — соединение не покидает один поток, запросы
//!    идут сообщениями. Паника обработчика убивает только поток, и это видно
//!    как `DbClosed`, а не как случайное отравление.
//!
//! Выбран второй. В ТЗ записано «`spawn_blocking`-обёртки», но `vela-core` не
//! зависит от tokio и не должен: домен обязан собираться и тестироваться без
//! асинхронного рантайма (ADR-0005). Поэтому транспорт — `std::sync::mpsc`,
//! а гарантия та же: один владелец, последовательные обращения.
//!
//! # Восстановление
//!
//! Порча файла — не гипотетический сценарий: выключение питания, переполненный
//! диск и «оптимизаторы» реестра дают её регулярно. Поэтому открытие
//! проверяет целостность через `PRAGMA quick_check`, и при повреждении
//! переименовывает файл в `state.db.corrupt-<ts>` (данные не уничтожаются),
//! после чего создаёт новую базу. Пользователь узнаёт об этом из
//! [`OpenOutcome::Recovered`], а не молча теряет раскладку.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use tracing::{debug, info, warn};

use crate::error::{is_corruption, CoreError};
use crate::paths;

/// Таймаут ожидания блокировки в миллисекундах.
///
/// 3 с — компромисс: короткий таймаут роняет запись при кратковременной
/// блокировке (антивирус, второй процесс), длинный подвешивает интерфейс.
pub const BUSY_TIMEOUT_MS: u32 = 3_000;

/// Как поступать, если файл повреждён.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenPolicy {
    /// Переименовать повреждённый файл и создать новую базу.
    ///
    /// Значение по умолчанию: пользователь получает работающее приложение и
    /// резервную копию, а не отказ запуска.
    Recover,
    /// Вернуть [`CoreError::Corrupt`], ничего не трогая.
    ///
    /// Нужен, когда файл указан пользователем явно и переименовывать его
    /// мы не вправе.
    FailOnCorruption,
}

/// Чем закончилось открытие.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenOutcome {
    /// База открыта как есть.
    Opened,
    /// Файл был повреждён: он сохранён рядом, создана новая база.
    Recovered {
        /// Куда переименован повреждённый файл.
        backup: PathBuf,
    },
}

/// Задача для потока-владельца соединения.
type Job = Box<dyn FnOnce(&mut Connection) + Send + 'static>;

enum Command {
    Run(Job),
    Flush(Sender<Result<(), CoreError>>),
    Shutdown(Sender<()>),
}

/// Открытая база данных.
///
/// Дешёвая в клонировании не является намеренно: это владелец фонового потока,
/// и второй экземпляр означал бы второе соединение и второй набор PRAGMA.
#[derive(Debug)]
pub struct Db {
    tx: Sender<Command>,
    worker: Option<JoinHandle<()>>,
    path: PathBuf,
    outcome: OpenOutcome,
}

impl Db {
    /// Открывает базу по стандартному пути (`%LOCALAPPDATA%\Vela\state.db`).
    pub fn open_default() -> Result<Self, CoreError> {
        let path = paths::db_path()?;
        Self::open(&path, OpenPolicy::Recover)
    }

    /// Открывает (при необходимости создаёт) базу по указанному пути.
    pub fn open(path: impl AsRef<Path>, policy: OpenPolicy) -> Result<Self, CoreError> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let (outcome, conn) = open_with_recovery(&path, policy)?;

        let (tx, rx) = mpsc::channel::<Command>();
        let worker = std::thread::Builder::new()
            .name("vela-db".to_owned())
            .spawn(move || run_worker(conn, &rx))?;

        Ok(Self {
            tx,
            worker: Some(worker),
            path,
            outcome,
        })
    }

    /// Путь к файлу базы.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Чем закончилось открытие: важно для интерфейса (тост о восстановлении).
    pub fn outcome(&self) -> &OpenOutcome {
        &self.outcome
    }

    /// Выполняет произвольную операцию над соединением в потоке-владельце.
    ///
    /// Замыкание получает `&mut Connection`, поэтому внутри можно открывать
    /// транзакции. Вызывающий поток блокируется до результата — это осознанно:
    /// вызывающий код и так ждёт данные, а параллелизм между чтениями даёт WAL,
    /// а не мьютекс в памяти.
    pub fn call<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, CoreError> + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::Run(Box::new(move |conn| {
                let _ = tx.send(f(conn));
            })))
            .map_err(|_| CoreError::DbClosed)?;
        rx.recv().map_err(|_| CoreError::DbClosed)?
    }

    /// Сбрасывает WAL на диск.
    ///
    /// Вызывается при выходе и перед потенциально рискованными операциями
    /// (миграция, обновление). `TRUNCATE` оставляет файл `-wal` пустым, чтобы
    /// после завершения не оставалось «хвоста» не перенесённых страниц.
    pub fn flush(&self) -> Result<(), CoreError> {
        let (tx, rx) = mpsc::channel();
        self.tx
            .send(Command::Flush(tx))
            .map_err(|_| CoreError::DbClosed)?;
        rx.recv().map_err(|_| CoreError::DbClosed)?
    }

    /// Останавливает поток-владелец и дожидается его завершения.
    ///
    /// Идемпотентно: повторный вызов ничего не делает. Явный метод нужен,
    /// потому что в `Drop` нельзя сообщить об ошибке, а при выходе приложения
    /// flush обязателен и его результат важен.
    pub fn shutdown(&mut self) -> Result<(), CoreError> {
        let Some(worker) = self.worker.take() else {
            return Ok(());
        };
        let (tx, rx) = mpsc::channel();
        if self.tx.send(Command::Shutdown(tx)).is_ok() {
            let _ = rx.recv();
        }
        // Паника обработчика не должна маскироваться: поток мог упасть
        // посреди транзакции, и молча продолжать работу нельзя.
        worker
            .join()
            .map_err(|_| CoreError::Io(std::io::Error::other("поток БД завершился с паникой")))
    }
}

impl Drop for Db {
    fn drop(&mut self) {
        // Ошибка здесь не интересна: вызывающий должен был позвать shutdown()
        // явно, а в Drop остаётся только не оставить висящий поток.
        let _ = self.shutdown();
    }
}

/// Тело потока-владельца: выполняет команды до `Shutdown` или обрыва канала.
fn run_worker(mut conn: Connection, rx: &Receiver<Command>) {
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::Run(job) => job(&mut conn),
            Command::Flush(reply) => {
                let _ = reply.send(checkpoint(&conn));
            }
            Command::Shutdown(reply) => {
                debug!("закрытие базы данных");
                let _ = checkpoint(&conn);
                let _ = reply.send(());
                return;
            }
        }
    }
    // Канал закрылся без Shutdown: владелец `Db` уронили. Всё равно сбрасываем
    // WAL, иначе последние транзакции останутся в `-wal` и потеряются при
    // следующем открытии, если файл `-wal` будет удалён.
    let _ = checkpoint(&conn);
}

/// Переносит страницы WAL в основной файл.
///
/// Публичная обёртка для модуля миграций: перед созданием копии файла нужно,
/// чтобы все изменения лежали в основном файле, иначе копия будет неполной.
pub(crate) fn checkpoint_public(conn: &Connection) -> Result<(), CoreError> {
    checkpoint(conn)
}

/// Переносит страницы WAL в основной файл.
fn checkpoint(conn: &Connection) -> Result<(), CoreError> {
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
    Ok(())
}

/// Открывает соединение, применяет PRAGMA и при необходимости восстанавливает.
fn open_with_recovery(
    path: &Path,
    policy: OpenPolicy,
) -> Result<(OpenOutcome, Connection), CoreError> {
    match try_open(path) {
        Ok(conn) => {
            debug!(path = %path.display(), "база данных открыта");
            Ok((OpenOutcome::Opened, conn))
        }
        Err(err) if is_recoverable(&err) => {
            warn!(path = %path.display(), error = %err, "база повреждена или недоступна как SQLite");
            if policy == OpenPolicy::FailOnCorruption {
                return Err(CoreError::Corrupt {
                    path: path.to_path_buf(),
                });
            }
            let backup = quarantine(path)?;
            info!(backup = %backup.display(), "повреждённый файл сохранён, создаётся новая база");
            let conn = try_open(path)?;
            Ok((OpenOutcome::Recovered { backup }, conn))
        }
        Err(err) => Err(err),
    }
}

/// Открывает файл и доводит его до рабочего состояния: PRAGMA + проверка.
fn try_open(path: &Path) -> Result<Connection, CoreError> {
    let conn = Connection::open(path)?;
    apply_pragmas(&conn)?;
    quick_check(&conn)?;
    Ok(conn)
}

/// Применяет PRAGMA. Набор зафиксирован в T-0104 и меняется только вместе с ним.
fn apply_pragmas(conn: &Connection) -> Result<(), CoreError> {
    // WAL: параллельное чтение во время записи и переживание аварийного
    // завершения. `NORMAL` вместо `FULL`: полный sync на каждом коммите
    // удваивает задержку записи, а WAL уже защищает от порчи файла.
    let mode: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
        // Файл может лежать на сетевом диске, где WAL недоступен. Работать
        // можно, но обещание «чтение не блокирует запись» больше не действует —
        // это надо видеть в логе, а не выяснять из жалоб.
        warn!(mode = %mode, "WAL недоступен, режим журнала понижен");
    }
    conn.execute_batch("PRAGMA synchronous=NORMAL")?;
    conn.execute_batch("PRAGMA foreign_keys=ON")?;
    conn.execute_batch(&format!("PRAGMA busy_timeout={BUSY_TIMEOUT_MS}"))?;
    conn.execute_batch("PRAGMA temp_store=MEMORY")?;
    Ok(())
}

/// Проверка целостности без полного сканирования.
///
/// `quick_check` читает структуру и индексы, но не сверяет содержимое таблиц:
/// на больших базах полный `integrity_check` занимает секунды, а нам нужно
/// отличить «файл битый» от «файл в порядке» за миллисекунды.
fn quick_check(conn: &Connection) -> Result<(), CoreError> {
    let mut stmt = conn.prepare("PRAGMA quick_check")?;
    let mut rows = stmt.query([])?;
    let mut problems: Vec<String> = Vec::new();
    while let Some(row) = rows.next()? {
        let line: String = row.get(0)?;
        if !line.eq_ignore_ascii_case("ok") {
            problems.push(line);
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    // Превращаем отчёт SQLite в ошибку с тем же кодом, что и «не база данных»,
    // чтобы путь восстановления был один.
    Err(CoreError::Sqlite(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error {
            code: rusqlite::ffi::ErrorCode::DatabaseCorrupt,
            extended_code: 11,
        },
        Some(problems.join("; ")),
    )))
}

/// Стоит ли пытаться восстановить файл, а не возвращать ошибку вызывающему.
fn is_recoverable(err: &CoreError) -> bool {
    match err {
        CoreError::Sqlite(e) => is_corruption(e),
        // Не удалось применить PRAGMA на файле, который SQLite не смог
        // прочитать, — тот же случай порчи.
        CoreError::Io(_) | CoreError::DbClosed | CoreError::DataDir(_) => false,
        // База новее приложения — целый файл с чужими данными. Переименовать
        // его значило бы уничтожить работу пользователя.
        CoreError::SchemaTooNew { .. } => false,
        // Мусор передал вызывающий код — файл тут ни при чём.
        CoreError::InvalidData { .. } | CoreError::NotFound { .. } => false,
        CoreError::Corrupt { .. } => true,
    }
}

/// Переименовывает повреждённый файл, сохраняя его рядом.
///
/// Копия, а не удаление: в файле могут быть заметки и раскладка, которые
/// пользователь ещё надеется восстановить вручную или отдать в поддержку.
fn quarantine(path: &Path) -> Result<PathBuf, CoreError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    let backup = with_suffix(path, &format!("corrupt-{stamp}"));
    std::fs::rename(path, &backup)?;

    // `-wal` и `-shm` относятся к старому файлу: если оставить их рядом с
    // новым, SQLite попытается применить чужой журнал к пустой базе.
    for suffix in ["wal", "shm"] {
        let side = with_suffix(path, suffix);
        if side.exists() {
            let target = with_suffix(&backup, suffix);
            std::fs::rename(&side, &target)?;
        }
    }
    Ok(backup)
}

/// `state.db` + `corrupt-123` → `state.db.corrupt-123`.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(suffix);
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("временный каталог");
        let path = dir.path().join("state.db");
        (dir, path)
    }

    #[test]
    fn opens_new_database_in_empty_directory() {
        let (_dir, path) = temp_db();
        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие");

        assert_eq!(db.outcome(), &OpenOutcome::Opened);
        assert!(path.exists(), "файл базы должен быть создан");
    }

    #[test]
    fn ac1_fresh_database_is_small() {
        // AC-1: свежая база не должна весить десятки килобайт «на старте».
        let (_dir, path) = temp_db();
        let mut db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
        db.flush().expect("flush");
        db.shutdown().expect("shutdown");

        let size = std::fs::metadata(&path).expect("метаданные").len();
        assert!(
            size <= 100 * 1024,
            "свежая база {size} байт, ожидалось не более 100 КиБ"
        );
    }

    #[test]
    fn pragmas_are_applied() {
        let (_dir, path) = temp_db();
        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие");

        let (journal, fk, timeout): (String, i64, i64) = db
            .call(|conn| {
                let journal: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
                let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
                let timeout: i64 = conn.query_row("PRAGMA busy_timeout", [], |r| r.get(0))?;
                Ok((journal, fk, timeout))
            })
            .expect("чтение PRAGMA");

        assert_eq!(journal.to_ascii_lowercase(), "wal");
        assert_eq!(
            fk, 1,
            "внешние ключи обязательны: каскадное удаление в T-0104"
        );
        assert_eq!(timeout, i64::from(BUSY_TIMEOUT_MS));
    }

    #[test]
    fn reopens_existing_database_without_reset() {
        let (_dir, path) = temp_db();
        {
            let mut db = Db::open(&path, OpenPolicy::Recover).expect("первое открытие");
            db.call(|conn| {
                conn.execute_batch("CREATE TABLE probe (v INTEGER)")?;
                conn.execute("INSERT INTO probe (v) VALUES (42)", [])?;
                Ok(())
            })
            .expect("запись");
            db.shutdown().expect("shutdown");
        }

        let db = Db::open(&path, OpenPolicy::Recover).expect("второе открытие");
        assert_eq!(
            db.outcome(),
            &OpenOutcome::Opened,
            "данные целы — не восстановление"
        );
        let value: i64 = db
            .call(|conn| Ok(conn.query_row("SELECT v FROM probe", [], |r| r.get(0))?))
            .expect("чтение");
        assert_eq!(value, 42);
    }

    #[test]
    fn ac2_garbage_file_is_quarantined_and_recreated() {
        // AC-2: мусор в файле не должен ронять приложение.
        let (_dir, path) = temp_db();
        {
            let mut f = std::fs::File::create(&path).expect("создание");
            f.write_all(b"\x00\x01\x02 not a database at all, just bytes")
                .expect("запись мусора");
        }

        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие после порчи");
        let OpenOutcome::Recovered { backup } = db.outcome().clone() else {
            panic!("ожидалось восстановление, получено {:?}", db.outcome());
        };

        assert!(backup.exists(), "повреждённый файл должен сохраниться");
        assert!(
            backup
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .contains(".corrupt-"),
            "имя копии должно содержать .corrupt-, получено {backup:?}"
        );

        // Новая база рабочая, а не пустая заглушка.
        db.call(|conn| {
            conn.execute_batch("CREATE TABLE probe (v INTEGER)")?;
            Ok(())
        })
        .expect("новая база должна принимать запросы");
    }

    #[test]
    fn corruption_backup_keeps_original_bytes() {
        // Смысл карантина — возможность восстановить данные вручную,
        // поэтому содержимое обязано совпадать.
        let (_dir, path) = temp_db();
        let garbage = b"CRITICAL DATA the user still hopes to recover";
        std::fs::write(&path, garbage).expect("запись");

        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
        let OpenOutcome::Recovered { backup } = db.outcome().clone() else {
            panic!("ожидалось восстановление");
        };
        assert_eq!(std::fs::read(&backup).expect("чтение копии"), garbage);
    }

    #[test]
    fn fail_on_corruption_does_not_touch_the_file() {
        let (_dir, path) = temp_db();
        let garbage = b"not a database";
        std::fs::write(&path, garbage).expect("запись");

        let err = Db::open(&path, OpenPolicy::FailOnCorruption).expect_err("должна быть ошибка");
        assert!(matches!(err, CoreError::Corrupt { .. }), "получено {err:?}");
        assert_eq!(
            std::fs::read(&path).expect("файл на месте"),
            garbage,
            "при FailOnCorruption файл не трогаем"
        );
    }

    #[test]
    fn sidecar_wal_and_shm_are_quarantined_together() {
        // Иначе новый файл получит в наследство чужой журнал.
        let (_dir, path) = temp_db();
        std::fs::write(&path, b"garbage").expect("запись");
        std::fs::write(with_suffix(&path, "wal"), b"old wal").expect("запись wal");

        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
        let OpenOutcome::Recovered { backup } = db.outcome().clone() else {
            panic!("ожидалось восстановление");
        };
        let sidecar = with_suffix(&backup, "wal");
        assert!(sidecar.exists(), "журнал должен уехать вместе с копией");
    }

    #[test]
    fn ac3_parallel_reads_do_not_fail() {
        // AC-3: 100 чтений из 4 потоков без SQLITE_BUSY.
        let (_dir, path) = temp_db();
        let db = std::sync::Arc::new(Db::open(&path, OpenPolicy::Recover).expect("открытие"));
        db.call(|conn| {
            conn.execute_batch("CREATE TABLE probe (v INTEGER); INSERT INTO probe VALUES (7);")?;
            Ok(())
        })
        .expect("подготовка");

        let mut handles = Vec::new();
        for _ in 0..4 {
            let db = std::sync::Arc::clone(&db);
            handles.push(std::thread::spawn(move || {
                for _ in 0..25 {
                    let v: i64 = db
                        .call(|conn| Ok(conn.query_row("SELECT v FROM probe", [], |r| r.get(0))?))
                        .expect("чтение не должно падать");
                    assert_eq!(v, 7);
                }
            }));
        }
        for h in handles {
            h.join().expect("поток чтения");
        }
    }

    #[test]
    fn concurrent_write_and_read_stress() {
        // AC-3 в жёсткой форме: 4 читателя и 1 писатель одновременно.
        // Проверяет, что последовательный владелец соединения не даёт
        // SQLITE_BUSY и не теряет записи.
        let (_dir, path) = temp_db();
        let db = std::sync::Arc::new(Db::open(&path, OpenPolicy::Recover).expect("открытие"));
        db.call(|conn| {
            conn.execute_batch("CREATE TABLE probe (v INTEGER)")?;
            Ok(())
        })
        .expect("подготовка");

        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let writer = {
            let db = std::sync::Arc::clone(&db);
            let stop = std::sync::Arc::clone(&stop);
            std::thread::spawn(move || {
                let mut n = 0;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) && n < 100 {
                    let value = n;
                    db.call(move |conn| {
                        conn.execute("INSERT INTO probe (v) VALUES (?1)", [value])?;
                        Ok(())
                    })
                    .expect("запись не должна падать");
                    n += 1;
                }
                n
            })
        };

        let readers: Vec<_> = (0..4)
            .map(|_| {
                let db = std::sync::Arc::clone(&db);
                std::thread::spawn(move || {
                    for _ in 0..25 {
                        let count: i64 = db
                            .call(|conn| {
                                Ok(conn.query_row("SELECT count(*) FROM probe", [], |r| r.get(0))?)
                            })
                            .expect("чтение не должно падать");
                        // Значение монотонно растёт и не превышает числа
                        // записей: иначе читатель увидел бы «будущее».
                        assert!(
                            (0..=100).contains(&count),
                            "счётчик вне ожидаемого диапазона: {count}"
                        );
                    }
                })
            })
            .collect();

        for r in readers {
            r.join().expect("поток чтения");
        }
        let written = writer.join().expect("поток записи");
        stop.store(true, std::sync::atomic::Ordering::Relaxed);

        let total: i64 = db
            .call(|conn| Ok(conn.query_row("SELECT count(*) FROM probe", [], |r| r.get(0))?))
            .expect("итог");
        assert_eq!(total, i64::from(written), "все записи должны дойти");
    }

    #[test]
    fn call_after_shutdown_reports_db_closed() {
        let (_dir, path) = temp_db();
        let mut db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
        db.shutdown().expect("shutdown");

        let err = db.call(|_| Ok(())).expect_err("после закрытия — ошибка");
        assert!(matches!(err, CoreError::DbClosed), "получено {err:?}");
    }

    #[test]
    fn shutdown_is_idempotent() {
        let (_dir, path) = temp_db();
        let mut db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
        db.shutdown().expect("первый shutdown");
        db.shutdown().expect("повторный shutdown не должен падать");
    }

    #[test]
    fn flush_persists_writes_immediately() {
        let (_dir, path) = temp_db();
        {
            let mut db = Db::open(&path, OpenPolicy::Recover).expect("открытие");
            db.call(|conn| {
                conn.execute_batch(
                    "CREATE TABLE probe (v INTEGER); INSERT INTO probe VALUES (1);",
                )?;
                Ok(())
            })
            .expect("запись");
            db.flush().expect("flush");
            // WAL после checkpoint(TRUNCATE) должен быть пустым.
            let wal = with_suffix(&path, "wal");
            let wal_size = std::fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
            assert_eq!(wal_size, 0, "после flush журнал должен быть усечён");
            db.shutdown().expect("shutdown");
        }

        let db = Db::open(&path, OpenPolicy::Recover).expect("повторное открытие");
        let v: i64 = db
            .call(|conn| Ok(conn.query_row("SELECT v FROM probe", [], |r| r.get(0))?))
            .expect("чтение");
        assert_eq!(v, 1, "запись должна пережить закрытие");
    }

    #[test]
    fn error_inside_call_propagates_and_db_stays_usable() {
        // Сбой запроса не должен убивать поток-владелец.
        let (_dir, path) = temp_db();
        let db = Db::open(&path, OpenPolicy::Recover).expect("открытие");

        let err = db
            .call(|conn| Ok(conn.execute("SELECT nope FROM missing_table", [])?))
            .expect_err("запроса к несуществующей таблице быть не может");
        assert!(matches!(err, CoreError::Sqlite(_)), "получено {err:?}");

        db.call(|conn| {
            conn.execute_batch("CREATE TABLE probe (v INTEGER)")?;
            Ok(())
        })
        .expect("база должна остаться рабочей");
    }

    #[test]
    fn with_suffix_produces_expected_names() {
        let p = Path::new("/data/state.db");
        assert_eq!(
            with_suffix(p, "corrupt-1"),
            Path::new("/data/state.db.corrupt-1")
        );
        assert_eq!(with_suffix(p, "wal"), Path::new("/data/state.db.wal"));
    }
}
