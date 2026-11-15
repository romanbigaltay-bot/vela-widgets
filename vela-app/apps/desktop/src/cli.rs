//! Разбор аргументов командной строки.
//!
//! Tauri и WebView2 передают свои аргументы (`--webview2-*`, путь к exe),
//! поэтому неизвестные флаги игнорируются, а не считаются ошибкой. Известные
//! флаги пока только запоминаются: их потребители появятся в T-0202
//! (`--no-scene`), T-0203 (`--safe-mode`) и T-0204 (`--settings`).

/// Флаги запуска, которые влияют на поведение приложения.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CliOptions {
    /// Открыть окно настроек при старте.
    pub open_settings: bool,
    /// Не создавать сцену (поверхность A).
    pub no_scene: bool,
    /// Безопасный режим: без WorkerW и хуков.
    pub safe_mode: bool,
}

impl CliOptions {
    /// Разбирает аргументы без учёта регистра не делает: флаги публичные
    /// и документированы в нижнем регистре.
    pub fn parse<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut options = Self::default();
        for arg in args {
            match arg.as_ref() {
                "--settings" => options.open_settings = true,
                "--no-scene" => options.no_scene = true,
                "--safe-mode" => options.safe_mode = true,
                _ => {}
            }
        }
        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_all_off() {
        assert_eq!(
            CliOptions::parse(Vec::<String>::new()),
            CliOptions::default()
        );
    }

    #[test]
    fn known_flags_are_recognised() {
        let options = CliOptions::parse(["--settings", "--no-scene", "--safe-mode"]);
        assert!(options.open_settings);
        assert!(options.no_scene);
        assert!(options.safe_mode);
    }

    #[test]
    fn unknown_arguments_are_ignored() {
        // Tauri/WebView2 передают свои аргументы; падать на них нельзя.
        let options = CliOptions::parse(["--webview2-chromium", "positional", "--settings"]);
        assert!(options.open_settings);
        assert!(!options.no_scene);
        assert!(!options.safe_mode);
    }

    #[test]
    fn repeated_flags_are_idempotent() {
        let options = CliOptions::parse(["--settings", "--settings"]);
        assert!(options.open_settings);
    }
}
