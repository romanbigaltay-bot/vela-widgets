//! Тонкая точка входа: вся логика живёт в библиотеке.
//!
//! Так `cargo test` проверяет те же модули, что и бинарь, а не отдельную
//! копию кода. `main` не содержит ни Tauri, ни обработки ошибок.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

fn main() -> std::process::ExitCode {
    vela::run()
}
