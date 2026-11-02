#![allow(clippy::unwrap_used, clippy::expect_used)] // тесты: паника = утверждение

//! Проверка зависимостей workspace.
//!
//! Правила графа (ST-0101-01, AC-2/AC-3):
//! 1. `vela-ipc` не зависит от `tauri` и `windows` — иначе типы контрактов
//!    нельзя проверить без графической среды.
//! 2. `vela-platform` не зависит от `tauri` — платформенный слой должен
//!    тестироваться отдельно от фреймворка окон.
//! 3. `vela-core` не зависит ни от одного крейта проекта — домен в основании.
//!
//! Проверяются манифесты на диске, а не `cargo metadata`: тест должен работать
//! без сети и без собранных зависимостей.

use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/vela-ipc
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root is two levels above the crate")
        .to_path_buf()
}

fn read_manifest(crate_dir: &str) -> String {
    let path = workspace_root().join(crate_dir).join("Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Имена зависимостей из `[dependencies]` и `[dev-dependencies]`
/// (без target-секций — они нужны только для проверки windows-слоя).
fn plain_dependencies(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // target-секции намеренно исключаем: они не входят в обычную сборку.
            in_deps = (trimmed == "[dependencies]" || trimmed == "[dev-dependencies]")
                || trimmed == "[build-dependencies]";
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            let key = key.split('.').next().unwrap_or("").trim();
            let key = key.trim_matches('"');
            if !key.is_empty() && !key.starts_with('#') {
                names.push(key.to_string());
            }
        }
    }
    names
}

fn has_any(names: &[String], forbidden: &[&str]) -> Vec<String> {
    names
        .iter()
        .filter(|n| forbidden.contains(&n.as_str()))
        .cloned()
        .collect()
}

#[test]
fn ipc_crate_is_framework_free() {
    let names = plain_dependencies(&read_manifest("crates/vela-ipc"));
    assert!(!names.is_empty(), "no dependencies found — parser broken");
    let bad = has_any(&names, &["tauri", "windows", "winit", "wgpu"]);
    assert!(
        bad.is_empty(),
        "vela-ipc must stay framework-free, found: {bad:?}"
    );
}

#[test]
fn platform_crate_does_not_depend_on_tauri() {
    let names = plain_dependencies(&read_manifest("crates/vela-platform"));
    let bad = has_any(&names, &["tauri"]);
    assert!(
        bad.is_empty(),
        "vela-platform must not depend on tauri, found: {bad:?}"
    );
}

#[test]
fn core_crate_has_no_project_dependencies() {
    let names = plain_dependencies(&read_manifest("crates/vela-core"));
    let bad = has_any(&names, &["vela-ipc", "vela-platform", "vela-cron", "vela"]);
    assert!(
        bad.is_empty(),
        "vela-core is the bottom of the graph and must not depend on project crates, found: {bad:?}"
    );
}

#[test]
fn platform_depends_on_core() {
    // Обратное направление разрешено явно: платформа знает домен, но не наоборот.
    let names = plain_dependencies(&read_manifest("crates/vela-platform"));
    assert!(
        names.iter().any(|n| n == "vela-core"),
        "vela-platform is expected to depend on vela-core, deps: {names:?}"
    );
}
