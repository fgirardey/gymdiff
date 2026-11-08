//! Keeps the Slint previews alive: every component, shell part, screen and the
//! app window has a `<name>.preview.slint` next to it, every preview compiles
//! with its gallery as the default component, and the app never imports them.
//!
//! See `docs/Workflow/Previews.md`.

use std::path::{Path, PathBuf};

use slint_interpreter::{Compiler, DiagnosticLevel};

/// Folders of `ui/` whose files must each have a preview.
const PREVIEWED_DIRS: [&str; 4] = ["components", "shell", "screens", "installer"];

/// Files at the root of `ui/` that must have a preview.
const PREVIEWED_ROOT_FILES: [&str; 2] = ["app-window.slint", "installer-window.slint"];

/// Folder of `ui/` with the preview helpers (`PreviewCase`, `Fixtures`, …).
const PREVIEW_KIT: &str = "preview-kit";

const PREVIEW_SUFFIX: &str = ".preview.slint";

/// Name of the gallery component every preview file ends with.
const GALLERY: &str = "Preview";

#[test]
fn every_component_shell_part_screen_and_the_app_window_has_a_preview() {
    let mut problems = Vec::new();

    for file in previewed_files() {
        if !preview_of(&file).exists() {
            problems.push(format!(
                "{}: add {}",
                display(&file),
                display(&preview_of(&file))
            ));
        }
    }
    for preview in preview_files() {
        if !component_of(&preview).exists() {
            problems.push(format!(
                "{}: no component file next to it",
                display(&preview)
            ));
        }
    }

    assert!(
        problems.is_empty(),
        "missing previews:\n{}",
        problems.join("\n")
    );
}

#[test]
fn every_preview_compiles_and_ends_with_its_gallery() {
    let mut failures = Vec::new();

    for file in preview_files() {
        let result = spin_on::spin_on(Compiler::default().build_from_path(&file));

        let errors: Vec<String> = result
            .diagnostics()
            .filter(|diagnostic| diagnostic.level() == DiagnosticLevel::Error)
            .map(|diagnostic| diagnostic.to_string())
            .collect();
        if !errors.is_empty() {
            failures.push(format!("{}: {}", display(&file), errors.join("; ")));
            continue;
        }

        // slint-viewer and `just preview` open the last exported component.
        // `component_names()` only lists windows, in no particular order, so read the source.
        let source = std::fs::read_to_string(&file).unwrap();
        let last = last_exported_component(&source);
        if last != Some(GALLERY) {
            failures.push(format!(
                "{}: the last exported component must be `{GALLERY}`, found {last:?}",
                display(&file)
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "broken previews:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_app_never_imports_previews_or_the_preview_kit() {
    let offenders: Vec<String> = all_slint_files()
        .into_iter()
        .filter(|file| !is_preview(file) && !file.starts_with(ui_dir().join(PREVIEW_KIT)))
        .filter(|file| {
            let source = std::fs::read_to_string(file).unwrap();
            source.contains(PREVIEW_SUFFIX) || source.contains(&format!("{PREVIEW_KIT}/"))
        })
        .map(|file| display(&file))
        .collect();

    assert!(
        offenders.is_empty(),
        "app files must not import previews or {PREVIEW_KIT}/: {offenders:?}"
    );
}

fn ui_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ui")
}

/// The files that must have a preview.
fn previewed_files() -> Vec<PathBuf> {
    let root_files = PREVIEWED_ROOT_FILES.iter().map(|name| ui_dir().join(name));
    let dir_files = PREVIEWED_DIRS
        .iter()
        .flat_map(|dir| slint_files(&ui_dir().join(dir)))
        .filter(|file| !is_preview(file));
    root_files.chain(dir_files).collect()
}

/// Every `*.preview.slint` of `ui/`.
fn preview_files() -> Vec<PathBuf> {
    all_slint_files()
        .into_iter()
        .filter(|file| is_preview(file))
        .collect()
}

/// Every `.slint` file of `ui/` and its direct sub-folders.
fn all_slint_files() -> Vec<PathBuf> {
    let mut files = slint_files(&ui_dir());
    for entry in std::fs::read_dir(ui_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(slint_files(&path));
        }
    }
    files
}

/// The `.slint` files directly in `dir`, sorted.
fn slint_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "slint"))
        .collect();
    files.sort();
    files
}

fn is_preview(file: &Path) -> bool {
    file_name(file).ends_with(PREVIEW_SUFFIX)
}

/// `screens/history.slint` → `screens/history.preview.slint`.
fn preview_of(file: &Path) -> PathBuf {
    let stem = file_name(file).trim_end_matches(".slint");
    file.with_file_name(format!("{stem}{PREVIEW_SUFFIX}"))
}

/// `screens/history.preview.slint` → `screens/history.slint`.
fn component_of(preview: &Path) -> PathBuf {
    let stem = file_name(preview).trim_end_matches(PREVIEW_SUFFIX);
    preview.with_file_name(format!("{stem}.slint"))
}

fn file_name(file: &Path) -> &str {
    file.file_name().and_then(|name| name.to_str()).unwrap()
}

/// Name in the last `export component <Name>` line of a `.slint` source.
fn last_exported_component(source: &str) -> Option<&str> {
    source
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("export component "))
        .filter_map(|rest| rest.split_whitespace().next())
        .next_back()
}

fn display(file: &Path) -> String {
    file.strip_prefix(ui_dir())
        .unwrap_or(file)
        .display()
        .to_string()
}
