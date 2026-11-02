//! Домен Vela: типы, доступ к данным, миграции.
//!
//! Крейт не знает ни про окна, ни про Tauri, ни про Win32. Это позволяет
//! тестировать бизнес-логику без графической среды (ср. ADR-0005).

// Тесты могут паниковать: `unwrap()` там — это утверждение об ожидаемом
// результате, а не обработка ошибки (см. clippy.toml, unwrap_used).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod db;
pub mod element;
pub mod error;
pub mod migrations;
pub mod paths;
pub mod repo;

pub use db::{Db, OpenOutcome, OpenPolicy, BUSY_TIMEOUT_MS};
pub use element::{Element, ElementKind, ElementPatch, PatchOp, Rect};
pub use error::CoreError;
pub use migrations::{migrate, migrate_with, MigrateOutcome, Migration, MIGRATIONS};
pub use paths::{data_dir, db_path, APP_DIR, DB_FILE};
pub use repo::{ElementsRepo, SettingsRepo, SqliteElements, SqliteSettings};

/// Идентификатор сборки домена — используется в диагностике и тестах
/// работоспособности workspace.
pub fn health() -> &'static str {
    "vela-core"
}

/// Версия схемы БД, которую понимает этот код.
///
/// Растёт при каждой миграции; используется при проверке совместимости
/// резервных копий (T-0104, ST-0104-02).
pub const SCHEMA_VERSION: u32 = 1;

// Компиляторный инвариант: код всегда знает хотя бы одну миграцию. Нулевая
// версия означала бы «схема не создана», и это должно ломать сборку.
const _: () = assert!(SCHEMA_VERSION >= 1);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_stable() {
        assert_eq!(health(), "vela-core");
    }

    // Версия схемы проверяется компиляторным `const _: () = assert!(...)` выше,
    // а не тестом: тест на константу не может упасть на рабочей сборке.
}
