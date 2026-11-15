//! Сборка Windows-ресурсов приложения.
//!
//! `tauri-build` встраивает манифест Common-Controls v6, иконку и метаданные
//! в `.exe`. На не-Windows он не нужен и не должен требовать Tauri-конфиг:
//! Linux CI собирает этот крейт как headless-заглушку.
//!
//! # Почему манифест линкуется в тесты отдельно
//!
//! `tauri-build` передаёт ресурс через `rustc-link-arg-bins`, то есть только
//! в `[[bin]]`. Тестовый бинарник — отдельный PE-файл; без Common-Controls v6
//! Windows грузит comctl32 v5, где нет `TaskDialogIndirect` (его импортирует
//! `tao`), и процесс падает с `STATUS_ENTRYPOINT_NOT_FOUND` до первого теста.
//! `embed-resource` умеет отдать тот же манифест именно тестовым целям.

fn main() {
    #[cfg(windows)]
    {
        // Манифест встраиваем сами, а не через `tauri-build`: он умеет
        // отдавать ресурс только бинарникам. `compile_for_everything` эмитит
        // `cargo:rustc-link-arg`, который применяется ко всем целям пакета,
        // включая unit-тесты в библиотеке.
        let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
        let attrs = tauri_build::Attributes::new().windows_attributes(windows);
        if let Err(err) = tauri_build::try_build(attrs) {
            panic!("tauri-build failed: {err}");
        }

        // `manifest_required` превращает отсутствие `windres` в ошибку сборки:
        // без манифеста тесты упадут на старте, и молча это оставлять нельзя.
        embed_resource::compile_for_everything("windows-app-manifest.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap_or_else(|err| panic!("{err}"));
    }

    // Конфиг, иконки и манифест — входные данные сборки: их изменение
    // обязано перезапускать `build.rs`, иначе в `.exe` останется старый ресурс.
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-changed=windows-app-manifest.rc");
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
}
