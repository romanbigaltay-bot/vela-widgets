//! Ошибки домена.
//!
//! Один тип на весь крейт: домен небольшой, и плодить по ошибке на модуль
//! значит заставлять вызывающего писать три вложенных `map_err`. Ветки
//! различаются по смыслу, а не по месту возникновения: `Sqlite` — сбой
//! библиотеки, `Io` — файловая система, `DbClosed` — обращение к уже
//! остановленной базе, `DataDir` — не удалось определить путь.
//!
//! Тексты сообщений короткие и машинно-читаемые: они попадают в лог, а не в
//! интерфейс (пользователю показывает `VelaError` из `vela-ipc`).

use thiserror::Error;

use crate::paths::DataDirError;

/// Ошибка операции домена.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Сбой SQLite.
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Сбой файловой системы.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// База уже остановлена: рабочий поток завершился или был закрыт явно.
    ///
    /// Отдельная ветка, а не `Io`: это ошибка порядка вызовов в коде, и по
    /// ней должно быть видно, что дело не в диске.
    #[error("база данных закрыта")]
    DbClosed,

    /// Не удалось определить каталог данных.
    #[error(transparent)]
    DataDir(#[from] DataDirError),

    /// Запрошенная сущность не найдена.
    ///
    /// Отличается от «пустого результата»: список пуст — это нормально,
    /// а изменение по несуществующему идентификатору — ошибка вызывающего кода.
    #[error("{entity} не найден: {id}")]
    NotFound {
        /// Вид сущности.
        entity: &'static str,
        /// Идентификатор, который искали.
        id: String,
    },

    /// Данные не прошли проверку домена.
    ///
    /// Отдельная ветка от `Sqlite`: вызывающий код должен различать «база
    /// отказала» и «мы сами передали мусор». Первое — повод повторить, второе —
    /// баг в коде, и повтор его не исправит.
    #[error("недопустимое поле {field}: {reason}")]
    InvalidData {
        /// Имя поля — чтобы ошибка указывала на место, а не на весь вызов.
        field: &'static str,
        /// Что именно не так.
        reason: String,
    },

    /// Схема в базе новее, чем понимает приложение.
    ///
    /// Возникает при откате на предыдущую версию: данные новых миграций есть,
    /// а кода, который их читает, нет. Понижать схему автоматически нельзя —
    /// это уничтожит данные, поэтому единственный корректный ответ — просить
    /// обновиться.
    #[error("схема базы ({found}) новее приложения ({supported})")]
    SchemaTooNew {
        /// Версия, найденная в базе.
        found: u32,
        /// Версия, которую понимает эта сборка.
        supported: u32,
    },

    /// SQLite сообщил, что файл повреждён.
    ///
    /// Возвращается, когда восстановление запрещено вызывающим кодом
    /// (`OpenPolicy::FailOnCorruption`) — например, при работе с файлом,
    /// который пользователь указал явно и мы не вправе переименовать.
    #[error("файл базы данных повреждён: {path}")]
    Corrupt {
        /// Путь к повреждённому файлу.
        path: std::path::PathBuf,
    },
}

/// Признак того, что ошибка означает повреждённый файл, а не сбой доступа.
///
/// Нужен, чтобы отличать «на диске мусор» от «нет прав»: первое лечится
/// переименованием, второе — нет, и предлагать пользователю «пересоздать
/// базу» в ответ на отказ доступа было бы вредительством.
pub(crate) fn is_corruption(err: &rusqlite::Error) -> bool {
    use rusqlite::ffi::ErrorCode;

    matches!(
        err,
        rusqlite::Error::SqliteFailure(e, msg)
            if matches!(
                e.code,
                ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase
            ) || msg
                .as_deref()
                .is_some_and(|m| m.contains("file is not a database"))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corruption_is_recognised() {
        let not_a_db = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::NotADatabase,
                extended_code: 26,
            },
            Some("file is not a database".to_owned()),
        );
        assert!(is_corruption(&not_a_db));

        let corrupt = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::DatabaseCorrupt,
                extended_code: 11,
            },
            None,
        );
        assert!(is_corruption(&corrupt));
    }

    #[test]
    fn access_errors_are_not_treated_as_corruption() {
        // Ключевое различие: отказ в доступе нельзя «лечить» переименованием
        // файла — данные целы, а пользователь потеряет их при пересоздании.
        let readonly = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::CannotOpen,
                extended_code: 8,
            },
            Some("unable to open database file".to_owned()),
        );
        assert!(!is_corruption(&readonly));

        let busy = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::DatabaseBusy,
                extended_code: 5,
            },
            None,
        );
        assert!(!is_corruption(&busy));
    }

    #[test]
    fn db_closed_message_is_human_readable() {
        assert_eq!(CoreError::DbClosed.to_string(), "база данных закрыта");
    }
}
