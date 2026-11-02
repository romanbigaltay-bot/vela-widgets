#![allow(clippy::unwrap_used, clippy::expect_used)] // тесты: паника = утверждение

//! Проверка архитектурных запретов для `vela-core`.
//!
//! Домен не должен зависеть от графического стека (ADR-0001) и от платформы:
//! иначе его нельзя протестировать без Windows и нельзя переиспользовать в
//! инструментах (например, в миграторе БД или в бенчмарке).
//!
//! # Почему проверка разделена на «чистую функцию» и «чтение манифеста»
//!
//! Первая версия теста читала `Cargo.toml` и падала только после того, как
//! cargo разрешит зависимости. Если нарушитель добавлял `tauri`, разрешение
//! падало раньше проверки, и тест молча не запускался — то есть правило
//! выглядело зелёным ровно в том случае, ради которого написано.
//!
//! Поэтому детектор — чистая функция [`violations`] над текстом манифеста.
//! Её можно проверить на подставном манифесте без сети и без сборки, а
//! реальный `Cargo.toml` подаётся в неё отдельным тестом.

const FORBIDDEN: &[&str] = &[
    "tauri",
    "tauri-build",
    "windows",
    "winit",
    "wgpu",
    "webkit2gtk",
    "gtk",
];

/// Возвращает имена зависимостей, объявленных в секциях `*dependencies*`.
///
/// Учитываются `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`
/// и target-секции (например, `[target.'cfg(windows)'.dependencies]`): для
/// домена все они одинаково запрещены.
pub fn dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_deps = false;

    for line in manifest.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_deps = trimmed.contains("dependencies");
            continue;
        }
        if !in_deps || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, _value)) = trimmed.split_once('=') else {
            continue;
        };
        // `serde = { version = "1" }` → `serde`
        // `ts-rs.workspace = true`      → `ts-rs`
        let key = key.trim();
        if key.is_empty() || key.contains(' ') {
            continue;
        }
        let base = key.split('.').next().unwrap_or(key).trim();
        let name = base.trim_matches('"').trim();
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }

    names
}

/// Возвращает запрещённые зависимости, найденные в манифесте.
///
/// Пустой результат означает «нарушений нет». Отдельная функция нужна, чтобы
/// тест мог проверить сам детектор на подставных данных.
pub fn violations(manifest: &str) -> Vec<String> {
    dependency_names(manifest)
        .into_iter()
        .filter(|name| FORBIDDEN.contains(&name.as_str()))
        .collect()
}

fn real_manifest() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    std::fs::read_to_string(path).expect("Cargo.toml must be readable")
}

#[test]
fn core_has_no_platform_or_ui_dependencies() {
    let manifest = real_manifest();
    let names = dependency_names(&manifest);

    // Защита от «зелёного» теста на пустом списке: в vela-core обязана быть
    // хотя бы одна зависимость, иначе парсер сломан и тест бесполезен.
    assert!(
        !names.is_empty(),
        "parser found no dependencies — the test would pass vacuously"
    );

    let found = violations(&manifest);
    assert!(
        found.is_empty(),
        "vela-core must not depend on platform/UI crates, found: {found:?}"
    );
}

#[test]
fn detector_catches_forbidden_dependency() {
    // Главный тест: правило обязано срабатывать. Без него весь файл — декорация.
    let manifest = r#"
[package]
name = "vela-core"

[dependencies]
serde = { version = "1" }
tauri = { version = "2" }
"#;
    assert_eq!(violations(manifest), vec!["tauri"]);
}

#[test]
fn detector_catches_forbidden_dependency_in_target_section() {
    // Win32-зависимость в target-секции — тоже нарушение для домена:
    // она ломает сборку домена на не-Windows.
    let manifest = r#"
[dependencies]
serde = "1"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58" }
"#;
    assert_eq!(violations(manifest), vec!["windows"]);
}

#[test]
fn detector_catches_build_and_dev_dependencies() {
    let manifest = r#"
[build-dependencies]
tauri-build = "2"

[dev-dependencies]
wgpu = "0.20"
"#;
    let mut found = violations(manifest);
    found.sort_unstable();
    assert_eq!(found, vec!["tauri-build", "wgpu"]);
}

#[test]
fn parser_ignores_package_fields_and_comments() {
    // `edition = "2021"` стоит до первой секции зависимостей и не должен
    // попасть в список; закомментированная зависимость — тоже.
    let manifest = r#"
[package]
name = "vela-core"
edition = "2021"

[dependencies]
# tauri = "2"
serde = { version = "1" }
ts-rs.workspace = true
"#;
    assert_eq!(dependency_names(manifest), vec!["serde", "ts-rs"]);
    assert!(violations(manifest).is_empty());
}
