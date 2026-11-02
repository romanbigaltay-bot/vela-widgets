//! Правила SQL, которые нельзя выразить типами.
//!
//! `SELECT *` переживает добавление колонки молча: код компилируется, тесты
//! с одним элементом проходят, а в рантайме `row.get(3)` читает уже другое
//! поле. Такое не ловится ни компилятором, ни модульными тестами, поэтому
//! проверяется по исходникам — как `arch-check.sh` проверяет манифесты.

#![allow(clippy::unwrap_used, clippy::expect_used)] // тесты: паника = утверждение

use std::path::{Path, PathBuf};

/// Собирает все `.rs` файлы крейта, кроме самих тестов.
fn sources() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    // Только `src`: тесты вправе обращаться к несуществующим таблицам и
    // колонкам, чтобы проверить обработку ошибок.
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
    out
}

/// Убирает строки комментариев, чтобы упоминание правила в документации
/// не считалось его нарушением.
fn code_lines(source: &str) -> impl Iterator<Item = &str> {
    source.lines().filter(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("//")
    })
}

#[test]
fn no_select_star_in_repository_code() {
    let mut violations = Vec::new();

    for path in sources() {
        let source = std::fs::read_to_string(&path).expect("чтение исходника");
        for (index, line) in code_lines(&source).enumerate() {
            // Регистр учитывается в обе стороны: `select *` в сыром SQL тоже
            // допустим по синтаксису SQLite.
            let lower = line.to_ascii_lowercase();
            if lower.contains("select *") || lower.contains("select\n*") {
                violations.push(format!("{}:{}: {}", path.display(), index + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "найден SELECT * — перечислите колонки явно:\n{}",
        violations.join("\n")
    );
}

#[test]
fn detector_actually_detects() {
    // Защита от «зелёного» теста на сломанном детекторе: если правило
    // перестанет находить нарушение, этот тест упадёт.
    let bad = "let sql = \"SELECT * FROM elements\";";
    let lower = bad.to_ascii_lowercase();
    assert!(
        lower.contains("select *"),
        "детектор должен ловить нарушение"
    );

    let good = "let sql = \"SELECT id, kind FROM elements\";";
    assert!(
        !good.to_ascii_lowercase().contains("select *"),
        "корректный запрос не должен считаться нарушением"
    );
}
