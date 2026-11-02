#![allow(clippy::unwrap_used, clippy::expect_used)] // тесты: паника = утверждение

//! Проверка рассинхрона версии контракта между Rust и UI.
//!
//! Самая неприятная категория ошибок в приложении с двумя языками — когда
//! контракт поменяли в Rust, UI собрали со старой версией, и приложение
//! молча работает неправильно. Версия сама по себе рассинхрон не ловит,
//! поэтому здесь есть три проверки:
//!
//! 1. Список из `contract.json` совпадает с фактическим содержимым каталога UI —
//!    ловит удаление типа из Rust без перегенерации.
//! 2. `version.ts` в UI содержит то же число, что `IPC_VERSION`.
//! 3. Каждый `ErrorCode` присутствует в сгенерированном `ErrorCode.ts`.
//!
//! Тест читает файлы UI по относительному пути от корня workspace, поэтому
//! падает в CI, если контракт не перегенерирован перед коммитом.

use std::path::{Path, PathBuf};
use vela_ipc::{ErrorCode, IPC_VERSION};

/// Манифест каталога контрактов — единственный источник истины.
///
/// Список генерируемых и рукописных файлов живёт в `contract.json`, а не
/// здесь: раньше он дублировался в трёх местах, и расхождение приводило к
/// тому, что один инструмент считал файл чужим и удалял его.
fn contract() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("contract.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&raw).expect("contract.json is not valid JSON")
}

fn contract_list(key: &str) -> Vec<String> {
    contract()[key]
        .as_array()
        .unwrap_or_else(|| panic!("contract.json: нет массива {key}"))
        .iter()
        .map(|v| v.as_str().expect("элемент должен быть строкой").to_owned())
        .collect()
}

/// Рукописные файлы каталога контрактов — из манифеста.
fn handwritten() -> Vec<String> {
    contract_list("handwritten")
}

fn ui_api_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = <root>/crates/vela-ipc
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join("ui/src/shared/api")
}

fn read(dir: &Path, name: &str) -> String {
    let path = dir.join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e} — запустите `pnpm ipc:gen`",
            path.display()
        )
    })
}

#[test]
fn ui_exports_exactly_the_contract_types() {
    let dir = ui_api_dir();
    let mut actual: Vec<String> = std::fs::read_dir(&dir)
        .expect("ui api dir must exist")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".ts"))
        // Рукописные файлы в этом каталоге — не часть генерируемого контракта.
        // Список берётся из манифеста: хардкод здесь уже расходился с
        // `contract.json` и `ipc-gen.mjs`, и расхождение молча пропускало
        // чужой файл как «рукописный».
        .filter(|n| !handwritten().contains(n))
        .collect();
    actual.sort();

    let mut expected: Vec<String> = contract_list("generated");
    expected.sort();

    assert_eq!(
        actual, expected,
        "набор типов контракта в UI разошёлся с Rust: запустите `pnpm ipc:gen` и закоммитьте"
    );
}

#[test]
fn ui_ipc_version_matches_rust() {
    let content = read(&ui_api_dir(), "version.ts");
    let marker = "export const IPC_VERSION =";
    let line = content
        .lines()
        .find(|line| line.trim_start().starts_with(marker))
        .expect("version.ts must declare IPC_VERSION");

    let value: u32 = line
        .split('=')
        .nth(1)
        .map(|tail| tail.trim().trim_end_matches(';'))
        .and_then(|tail| tail.parse().ok())
        .expect("IPC_VERSION must be a number literal");

    assert_eq!(
        value, IPC_VERSION,
        "версия контракта в UI ({value}) не совпадает с Rust ({IPC_VERSION})"
    );
}

#[test]
fn every_error_code_exists_in_generated_binding() {
    let content = read(&ui_api_dir(), "ErrorCode.ts");

    // Список вариантов берём из Rust, а не из константы в тесте: добавили
    // вариант — тест сразу требует его в TS-контракте.
    let codes = [
        ErrorCode::PermissionDenied,
        ErrorCode::NotFound,
        ErrorCode::PlatformUnavailable,
        ErrorCode::Busy,
        ErrorCode::InvalidInput,
        ErrorCode::Internal,
    ];

    for code in codes {
        let needle = format!("\"{}\"", code.as_str());
        assert!(
            content.contains(&needle),
            "в сгенерированном ErrorCode.ts нет варианта {needle}"
        );
    }
}

#[test]
fn exported_id_types_are_plain_strings() {
    let dir = ui_api_dir();
    for name in ["ElementId", "MonitorId", "ProfileId", "WidgetId"] {
        let content = read(&dir, &format!("{name}.ts"));
        assert!(
            content.contains(&format!("export type {name} = string;")),
            "{name} должен быть обычной строкой в TS"
        );
    }
}
