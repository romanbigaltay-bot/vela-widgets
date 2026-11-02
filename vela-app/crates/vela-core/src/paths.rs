//! Пути пользовательских данных.
//!
//! Единственное место, где раскладывается `%LOCALAPPDATA%\Vela`. Отдельный
//! модуль, а не константа в `db`, потому что тот же путь понадобится кэшу
//! снимков (P5), логам (T-0805) и обновлятору (T-0803), и разъезжаться им
//! нельзя: приложение, которое хранит данные в двух местах, невозможно
//! корректно удалить.
//!
//! Функции принимают источник переменных окружения параметром — так их можно
//! проверить без правки реального окружения процесса (`std::env::set_var`
//! в тестах гоняется между потоками и делает прогон недетерминированным).

use std::path::PathBuf;

/// Имя каталога приложения внутри пользовательских данных.
pub const APP_DIR: &str = "Vela";

/// Имя файла базы данных.
pub const DB_FILE: &str = "state.db";

/// Каталог данных приложения.
///
/// * Windows: `%LOCALAPPDATA%\Vela`.
/// * Остальные системы: `$XDG_DATA_HOME/Vela` либо `~/.local/share/Vela`.
///   Ветка существует не ради поддержки других ОС, а ради того, чтобы тесты
///   домена шли в Linux-джобе CI.
#[derive(Debug)]
pub struct DataDirError {
    /// Переменные, которых не хватило для определения пути.
    pub checked: &'static str,
}

impl std::fmt::Display for DataDirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "не удалось определить каталог данных: не заданы переменные окружения {}",
            self.checked
        )
    }
}

impl std::error::Error for DataDirError {}

/// Возвращает каталог данных, читая реальное окружение процесса.
pub fn data_dir() -> Result<PathBuf, DataDirError> {
    data_dir_from(|key| std::env::var_os(key).map(PathBuf::from))
}

/// Полный путь к файлу базы данных.
pub fn db_path() -> Result<PathBuf, DataDirError> {
    Ok(data_dir()?.join(DB_FILE))
}

/// Вычисляет каталог данных по произвольному источнику переменных окружения.
fn data_dir_from(get: impl Fn(&str) -> Option<PathBuf>) -> Result<PathBuf, DataDirError> {
    #[cfg(windows)]
    {
        if let Some(base) = get("LOCALAPPDATA") {
            return Ok(base.join(APP_DIR));
        }
        // Фолбэк для урезанных профилей, где LOCALAPPDATA не выставлен.
        if let Some(home) = get("USERPROFILE") {
            return Ok(home.join("AppData").join("Local").join(APP_DIR));
        }
        Err(DataDirError {
            checked: "LOCALAPPDATA, USERPROFILE",
        })
    }

    #[cfg(not(windows))]
    {
        if let Some(base) = get("XDG_DATA_HOME") {
            return Ok(base.join(APP_DIR));
        }
        if let Some(home) = get("HOME") {
            return Ok(home.join(".local").join("share").join(APP_DIR));
        }
        Err(DataDirError {
            checked: "XDG_DATA_HOME, HOME",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_dir_name_is_stable() {
        // Имя каталога — часть контракта с установщиком и удалятором (T-0803):
        // смена строки осиротит данные у уже установленных копий.
        assert_eq!(APP_DIR, "Vela");
        assert_eq!(DB_FILE, "state.db");
    }

    #[test]
    fn db_file_lives_inside_app_dir() {
        let path = PathBuf::from("/base/Vela/state.db");
        assert!(path.ends_with(PathBuf::from(APP_DIR).join(DB_FILE)));
    }

    #[cfg(windows)]
    #[test]
    fn uses_localappdata_on_windows() {
        let dir = data_dir_from(|key| match key {
            "LOCALAPPDATA" => Some(PathBuf::from(r"C:\Users\u\AppData\Local")),
            _ => None,
        })
        .expect("LOCALAPPDATA задан");
        assert!(dir.ends_with(APP_DIR));
        assert!(dir.starts_with(r"C:\Users\u\AppData\Local"));

        // Без LOCALAPPDATA используется USERPROFILE, а не текущий каталог:
        // писать данные рядом с exe нельзя — Program Files только для чтения.
        let dir = data_dir_from(|key| match key {
            "USERPROFILE" => Some(PathBuf::from(r"C:\Users\u")),
            _ => None,
        })
        .expect("USERPROFILE задан");
        assert!(dir.starts_with(r"C:\Users\u\AppData\Local"));
    }

    #[cfg(not(windows))]
    #[test]
    fn uses_xdg_or_home_on_unix() {
        let dir = data_dir_from(|key| match key {
            "XDG_DATA_HOME" => Some(PathBuf::from("/xdg")),
            "HOME" => Some(PathBuf::from("/home/u")),
            _ => None,
        })
        .expect("XDG_DATA_HOME задан");
        assert_eq!(dir, PathBuf::from("/xdg").join(APP_DIR));

        let dir = data_dir_from(|key| match key {
            "HOME" => Some(PathBuf::from("/home/u")),
            _ => None,
        })
        .expect("HOME задан");
        assert_eq!(dir, PathBuf::from("/home/u/.local/share").join(APP_DIR));
    }

    #[test]
    fn missing_environment_is_an_error_not_a_default() {
        // Молчаливая запись в текущий каталог хуже явной ошибки: пользователь
        // не понимает, откуда взялся state.db в папке с игрой.
        let err = data_dir_from(|_| None).expect_err("ни одна переменная не задана");
        assert!(!err.checked.is_empty());
        assert!(err.to_string().contains("каталог данных"));
    }
}
