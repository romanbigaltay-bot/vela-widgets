//! Открытие базы и миграции до запуска окон.
//!
//! Модуль отделён от `lib.rs`, чтобы порядок «сначала данные, потом UI» был
//! виден в одном месте, а не размазан по точке входа.

use vela_core::{Db, MigrateOutcome, OpenOutcome};

use crate::exit_code::ExitCodeVela;

/// База, готовая к работе.
pub struct Prepared {
    db: Db,
}

impl Prepared {
    /// Останавливает поток-владелец базы и дожидается его завершения.
    pub fn shutdown(mut self) -> Result<(), vela_core::CoreError> {
        self.db.shutdown()
    }
}

/// Открывает базу, восстанавливает её при необходимости и применяет миграции.
///
/// Возвращает код ошибки, а не `anyhow::Error`: вызывающему нужно различать
/// «база новее бинаря» и «база сломана», а текст ошибки — только для лога.
pub fn prepare() -> Result<Prepared, ExitCodeVela> {
    let mut db = match Db::open_default() {
        Ok(db) => db,
        Err(err) => {
            tracing::error!(error = %err, "не удалось открыть базу данных");
            return Err(ExitCodeVela::InitFailed);
        }
    };

    // Восстановление после порчи — не ошибка запуска: приложение обязано
    // работать, а пользователь узнать, что старый файл сохранён рядом.
    if let OpenOutcome::Recovered { backup } = db.outcome() {
        tracing::warn!(
            backup = %backup.display(),
            "база была повреждена: файл сохранён, создана новая"
        );
    }

    match vela_core::migrate(&db) {
        Ok(MigrateOutcome::Applied { from, to, backup }) => {
            tracing::info!(from, to, backup = ?backup, "схема обновлена");
        }
        // UpToDate намеренно не логируется: библиотека уже сообщила это
        // на уровне debug, а повтор из бинаря только удваивает строки.
        Ok(MigrateOutcome::UpToDate { .. }) => {}
        Err(vela_core::CoreError::SchemaTooNew { found, supported }) => {
            // Приложение старше базы: это результат отката на предыдущую
            // версию. Понижать схему нельзя, поэтому сообщаем и выходим
            // отдельным кодом, чтобы обновлятор это увидел.
            tracing::error!(found, supported, "база создана более новой версией");
            let _ = db.shutdown();
            return Err(ExitCodeVela::SchemaTooNew);
        }
        Err(err) => {
            tracing::error!(error = %err, "не удалось применить миграции");
            let _ = db.shutdown();
            return Err(ExitCodeVela::InitFailed);
        }
    }

    Ok(Prepared { db })
}
