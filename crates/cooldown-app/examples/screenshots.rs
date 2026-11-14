//! Renders the Pencil stories of the Slint previews to PNG, headless, for pull
//! requests: `just screenshots [ui paths…]`.
//!
//! A Pencil story is an exported component right after a "// Pencil frame `…`"
//! comment in a `*.preview.slint` file (see `docs/Workflow/Previews.md`). Each one
//! is drawn by Slint's software renderer at 2× and saved as
//! `<out>/<preview path with dashes>--<Story>.png`; `<out>/index.tsv` lists
//! `file<TAB>frame<TAB>preview` (path under `ui/`) for each image.
//!
//! Arguments: `--out <dir>` (default `target/screenshots`), then the preview
//! files to render, relative to `ui/` (e.g. `screens/welcome`); none renders all.
//!
//! See `docs/Workflow/Pull requests.md`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result, bail};
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter, WindowEvent};
use slint::{ComponentHandle, PhysicalSize};
use slint_interpreter::{Compiler, ComponentInstance, DiagnosticLevel, Value};

const SCALE: f32 = 2.0;
const PREVIEW_SUFFIX: &str = ".preview.slint";
const PENCIL_MARKER: &str = "// Pencil frame `";

fn main() -> Result<()> {
    let (out_dir, filters) = parse_args()?;
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(HeadlessPlatform {
        window: window.clone(),
    }))
    .map_err(|error| anyhow::anyhow!("cannot set the Slint platform: {error}"))?;

    std::fs::create_dir_all(&out_dir)?;
    let mut index = String::new();
    for preview in preview_files(&filters)? {
        let source = std::fs::read_to_string(&preview)?;
        for story in pencil_stories(&source) {
            let file_name = format!("{}--{}.png", slug(&preview), story.component);
            render(
                &window,
                &preview,
                &story.component,
                &out_dir.join(&file_name),
            )
            .with_context(|| format!("{}: {}", display(&preview), story.component))?;
            writeln!(
                index,
                "{file_name}\t{}\t{}",
                story.frame,
                relative(&preview)
            )?;
            println!("{}", out_dir.join(&file_name).display());
        }
    }
    std::fs::write(out_dir.join("index.tsv"), index)?;
    Ok(())
}

/// A component of a preview file that reproduces a Pencil frame.
struct Story {
    frame: String,
    component: String,
}

/// Lets the interpreter create its window without opening one on screen.
struct HeadlessPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for HeadlessPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
}

fn parse_args() -> Result<(PathBuf, Vec<String>)> {
    let mut out_dir = workspace_dir().join("target/screenshots");
    let mut filters = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--out" {
            out_dir = PathBuf::from(args.next().context("--out needs a directory")?);
        } else {
            filters.push(
                arg.trim_end_matches(PREVIEW_SUFFIX)
                    .trim_end_matches(".slint")
                    .to_owned(),
            );
        }
    }
    Ok((out_dir, filters))
}

/// Wraps the story in a window sized to it, renders it and writes the PNG.
fn render(window: &MinimalSoftwareWindow, preview: &Path, story: &str, png: &Path) -> Result<()> {
    let file_name = preview.file_name().unwrap().to_string_lossy();
    let wrapper = format!(
        "import {{ {story} }} from \"{file_name}\";\n\
         export component Screenshot inherits Window {{\n\
         \x20   out property <length> story-width: story.width;\n\
         \x20   out property <length> story-height: story.height;\n\
         \x20   story := {story} {{ }}\n\
         }}\n"
    );
    // The wrapper lives next to the preview so that relative imports resolve.
    let wrapper_path = preview.with_file_name("__screenshot__.slint");
    let result = spin_on::spin_on(Compiler::default().build_from_source(wrapper, wrapper_path));
    let errors: Vec<String> = result
        .diagnostics()
        .filter(|diagnostic| diagnostic.level() == DiagnosticLevel::Error)
        .map(|diagnostic| diagnostic.to_string())
        .collect();
    if !errors.is_empty() {
        bail!("does not compile: {}", errors.join("; "));
    }
    let instance = result
        .component("Screenshot")
        .context("no Screenshot component")?
        .create()?;

    window.dispatch_event(WindowEvent::ScaleFactorChanged {
        scale_factor: SCALE,
    });
    let width = logical_length(&instance, "story-width")?;
    let height = logical_length(&instance, "story-height")?;
    if width <= 0.0 || height <= 0.0 {
        bail!("has no size: give the story a fixed width and height");
    }
    // Window sizes are small positive pixel counts.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let size = PhysicalSize::new(
        (width * SCALE).round() as u32,
        (height * SCALE).round() as u32,
    );
    window.set_size(size);
    instance.show()?;

    let snapshot = instance.window().take_snapshot()?;
    instance.hide()?;
    write_png(
        png,
        snapshot.width(),
        snapshot.height(),
        snapshot.as_bytes(),
    )
}

fn logical_length(instance: &ComponentInstance, property: &str) -> Result<f32> {
    match instance.get_property(property)? {
        // Slint lengths are f32; the interpreter widens them to f64.
        #[allow(clippy::cast_possible_truncation)]
        Value::Number(value) => Ok(value as f32),
        other => bail!("{property} is not a length: {other:?}"),
    }
}

fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(rgba)?;
    Ok(())
}

/// The Pencil stories of a preview file, in source order.
fn pencil_stories(source: &str) -> Vec<Story> {
    let mut stories = Vec::new();
    let mut lines = source.lines().map(str::trim);
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix(PENCIL_MARKER) else {
            continue;
        };
        let Some((frame, _)) = rest.split_once('`') else {
            continue;
        };
        let component = lines
            .next()
            .and_then(|next| next.strip_prefix("export component "))
            .and_then(|next| next.split_whitespace().next());
        if let Some(component) = component {
            stories.push(Story {
                frame: frame.to_owned(),
                component: component.to_owned(),
            });
        }
    }
    stories
}

/// Every `*.preview.slint` file of `ui/`, or only those named in `filters`.
fn preview_files(filters: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_previews(&ui_dir(), &mut files)?;
    files.sort();
    if !filters.is_empty() {
        files.retain(|file| filters.iter().any(|filter| relative(file) == *filter));
    }
    Ok(files)
}

fn collect_previews(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_previews(&path, files)?;
        } else if path.to_string_lossy().ends_with(PREVIEW_SUFFIX) {
            files.push(path);
        }
    }
    Ok(())
}

/// `ui/screens/welcome.preview.slint` → `screens/welcome`.
fn relative(preview: &Path) -> String {
    let path = preview.strip_prefix(ui_dir()).unwrap_or(preview);
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches(PREVIEW_SUFFIX)
        .to_owned()
}

/// `ui/screens/welcome.preview.slint` → `screens-welcome`.
fn slug(preview: &Path) -> String {
    relative(preview).replace('/', "-")
}

fn display(path: &Path) -> String {
    path.strip_prefix(workspace_dir())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn ui_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn workspace_dir() -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    dir.canonicalize().unwrap_or(dir)
}
