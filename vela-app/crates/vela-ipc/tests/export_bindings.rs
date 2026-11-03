#![allow(clippy::unwrap_used, clippy::expect_used)] // тесты: паника = утверждение

//! Тесты генерации контракта для UI.
//!
//! Запускаются только с фичей `ts-export`:
//! `TS_RS_EXPORT_DIR=... cargo test -p vela-ipc --features ts-export --test export_bindings`.
//! Штатный способ — `pnpm ipc:gen` из каталога `ui/`.
//!
//! # Почему тест, а не build-скрипт
//!
//! Build-скрипт выполнялся бы при каждой сборке (в том числе у пользователя
//! при `cargo install`) и писал бы файлы в чужой каталог. Тест запускается
//! явно — ровно тогда, когда контракт нужно обновить.
//!
//! # Почему проверяется список файлов
//!
//! `ts-rs` пишет по файлу на тип. Если тип переименовали, старый файл
//! останется и продолжит импортироваться — получится «мёртвый» контракт,
//! который выглядит рабочим.

#![cfg(feature = "ts-export")]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// Манифест каталога контрактов — единственный источник истины.
///
/// Раньше список рукописных файлов дублировался в трёх местах (здесь, в
/// `contract_sync.rs` и в `ui/scripts/ipc-gen.mjs`), и каждое расхождение
/// заканчивалось удалением файла из рабочего дерева. Теперь список один.
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

/// Типы, которые обязаны быть сгенерированы.
fn expected() -> Vec<String> {
    contract_list("generated")
}

/// Рукописные файлы: генерация их не создаёт и не удаляет.
fn handwritten() -> Vec<String> {
    contract_list("handwritten")
}

fn is_handwritten(name: &str) -> bool {
    handwritten().iter().any(|h| h == name)
}

/// Каталог, куда `ts-rs` пишет типы, если он задан.
///
/// Относительный путь обязан резолвиться от корня workspace, а не от текущего
/// каталога теста. Иначе `TS_RS_EXPORT_DIR=ui/src/shared/api` создаёт
/// `crates/vela-ipc/ui/src/shared/api/` — мусорную копию контракта, которая
/// выглядит как настоящая и сбивает с толку при разборе.
///
/// Уже абсолютный путь принимается как есть: `ipc-gen.mjs` передаёт именно
/// такой. Без переменной возвращается `None`: прямой `cargo test --all-features`
/// не должен падать только потому, что ему не сказали, куда писать.
fn export_dir() -> Option<PathBuf> {
    let raw = std::env::var("TS_RS_EXPORT_DIR").ok()?;
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return Some(path);
    }
    Some(workspace_root().join(path))
}

/// Корень репозитория: `<root>/crates/vela-ipc` → `<root>`.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn export_all(dir: &Path) {
    // `export_all_to` пишет тип вместе с зависимостями: достаточно перечислить
    // корни контракта, вложенные типы уедут сами.
    vela_ipc::ElementId::export_all_to(dir).expect("export ElementId");
    vela_ipc::MonitorId::export_all_to(dir).expect("export MonitorId");
    vela_ipc::ProfileId::export_all_to(dir).expect("export ProfileId");
    vela_ipc::WidgetId::export_all_to(dir).expect("export WidgetId");
    vela_ipc::ErrorCode::export_all_to(dir).expect("export ErrorCode");
    vela_ipc::VelaError::export_all_to(dir).expect("export VelaError");
    // Envelope generic: ts-rs не выводит generic-типы сам, поэтому контракт
    // объявляет явное инстанцирование (`UnitEnvelope`). В TS получается
    // `Envelope<T>`, который UI подставляет конкретными типами.
    vela_ipc::UnitEnvelope::export_all_to(dir).expect("export Envelope");
}

fn generated_files(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .expect("export dir must exist")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".ts") && !is_handwritten(name))
        .collect()
}

#[test]
fn generated_set_matches_expected() {
    // Проверка идёт во ВРЕМЕННОМ каталоге, а не в реальном.
    //
    // Изначально этот тест писал и удалял файлы прямо в `ui/src/shared/api`,
    // чтобы «почистить стейл». Это оказалось дефектом: `pnpm ipc:check`
    // вызывает тот же экспорт, и проверка типов молча удаляла из рабочего
    // дерева любой файл, которого не было в EXPECTED/HANDWRITTEN — включая
    // только что написанные рукописные модули. Инструмент проверки не должен
    // менять то, что проверяет.
    //
    // Стейл-файлы теперь ловит отдельный тест `no_stray_contract_files`,
    // который только читает и потому безопасен.
    let dir = tempfile::tempdir().expect("create temp dir");
    let dir = dir.path();
    export_all(dir);

    let expected: BTreeSet<String> = expected().into_iter().collect();
    assert_eq!(
        generated_files(dir),
        expected,
        "набор сгенерированных типов разошёлся с ожидаемым: тип добавлен или \
         переименован в src/ без обновления EXPECTED"
    );
}

#[test]
fn handwritten_files_are_never_touched() {
    // Критично: проверка идёт во ВРЕМЕННОМ каталоге, а не в реальном.
    // Первая версия теста писала пробы в `ui/src/shared/api` и удаляла их
    // вместе с настоящим `version.ts` — то есть тест уничтожал контракт,
    // который проверял.
    let dir = tempfile::tempdir().expect("create temp dir");
    let dir = dir.path();

    let probe_content = "// probe\n";
    for name in handwritten() {
        std::fs::write(dir.join(name), probe_content).expect("write probe");
    }

    export_all(dir);

    for name in handwritten() {
        let path = dir.join(&name);
        assert!(
            path.exists(),
            "рукописный файл {name} был удалён генерацией — это ломает UI"
        );
        let content = std::fs::read_to_string(&path).expect("read probe");
        assert_eq!(
            content, probe_content,
            "рукописный файл {name} был перезаписан генерацией"
        );
    }
}

#[test]
fn no_stray_contract_files() {
    // Ловит ровно ту ошибку, которую механический тест выше пропускает:
    // файл, которого нет ни в EXPECTED, ни в HANDWRITTEN, будет считаться
    // сгенерированным и удалён при первом же `pnpm ipc:gen`.
    //
    // `handwritten_files_are_never_touched` проверяет механизм («файлы из
    // списка не трогаются»), но не полноту списка — поэтому новый рукописный
    // файл проходил мимо него и ломался только в CI на JS-проверке.
    // Без TS_RS_EXPORT_DIR проверять нечего: прямой `cargo test` не должен
    // падать. Реальную проверку выполняет `pnpm ipc:check`, который задаёт
    // переменную и сравнивает настоящий каталог контрактов.
    let Some(dir) = export_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        // Контракт ещё не сгенерирован: проверять нечего, и это не ошибка
        // (чистый клон до первого `pnpm ipc:gen`).
        return;
    };

    let expected: BTreeSet<String> = expected().into_iter().collect();
    let mut stray: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".ts") {
            continue;
        }
        if expected.contains(&name) || is_handwritten(&name) {
            continue;
        }
        stray.push(name);
    }

    assert!(
        stray.is_empty(),
        "в каталоге контрактов есть файлы вне EXPECTED и HANDWRITTEN: {stray:?}.\n\
         Они будут удалены при следующем `pnpm ipc:gen` — добавьте их в один из списков."
    );
}

#[test]
fn id_bindings_are_plain_strings() {
    // Проверка идёт во временном каталоге: тест не должен менять файлы,
    // которые проверяет, и не должен зависеть от TS_RS_EXPORT_DIR.
    let dir = tempfile::tempdir().expect("create temp dir");
    export_all(dir.path());

    let content =
        std::fs::read_to_string(dir.path().join("ElementId.ts")).expect("ElementId binding");
    assert!(
        content.contains("export type ElementId = string;"),
        "ElementId должен быть обычной строкой в TS, получено:\n{content}"
    );
}

#[test]
fn error_code_binding_is_string_union() {
    // Тот же принцип: временный каталог, реальные контракты не трогаем.
    let dir = tempfile::tempdir().expect("create temp dir");
    export_all(dir.path());

    let content =
        std::fs::read_to_string(dir.path().join("ErrorCode.ts")).expect("ErrorCode binding");
    // UI обрабатывает коды как строковые литералы; числовой enum сломал бы
    // сопоставление в switch и в записях лога.
    for code in [
        "permission_denied",
        "not_found",
        "platform_unavailable",
        "busy",
        "invalid_input",
        "internal",
    ] {
        assert!(
            content.contains(&format!("\"{code}\"")),
            "ErrorCode должен содержать литерал {code}, получено:\n{content}"
        );
    }
}
