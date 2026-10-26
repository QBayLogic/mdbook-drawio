use anyhow::Result;
use log::{debug, error, info};
use mdbook_preprocessor::book::{Book, Chapter};
use mdbook_preprocessor::errors::Error;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Instant, UNIX_EPOCH};

mod directive;
mod steps;

pub use directive::{find_directives, Directive, Steps};

pub struct DrawioPreprocessor;

impl Preprocessor for DrawioPreprocessor {
    fn name(&self) -> &str {
        "mdbook-drawio"
    }

    fn supports_renderer(&self, _renderer: &str) -> Result<bool, anyhow::Error> {
        Ok(true)
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let config = Config::from_context(ctx)?;

        // Replace the directives with image links, and collect the SVGs they
        // need. The links do not depend on the export, so all exports can run
        // afterwards, in parallel.
        let mut exports = BTreeMap::new();
        let mut errors = Vec::new();
        book.for_each_chapter_mut(|ch| {
            if let Err(e) = process_chapter(ctx, &config, ch, &mut exports) {
                let name = ch.path.as_deref().unwrap_or(Path::new(&ch.name));
                errors.push(format!("{}: {e}", name.display()));
            }
        });
        if !errors.is_empty() {
            return Err(Error::msg(errors.join("\n")));
        }

        let stale: Vec<Export> = exports.into_values().filter(|e| e.stale).collect();
        export_all(&config, &stale)?;
        Ok(book)
    }
}

/// The settings from the `[preprocessor.drawio]` table in `book.toml`.
struct Config {
    /// Absolute path of the book's `src` directory.
    src: PathBuf,
    /// Where the SVGs are written, relative to `src`.
    result_dir: PathBuf,
    drawio_bin: String,
    drawio_args: Vec<String>,
    /// The number of draw.io processes to run at once.
    jobs: usize,
    /// Identifies the draw.io installation, see [`drawio_identity`].
    drawio_identity: String,
}

impl Config {
    fn from_context(ctx: &PreprocessorContext) -> Result<Config, Error> {
        let drawio_bin = get_preprocessor_value(ctx, "drawio-bin", "drawio".to_string());
        let default_jobs = std::thread::available_parallelism().map_or(1, |n| n.get().min(8));
        Ok(Config {
            src: ctx.root.join(&ctx.config.book.src),
            result_dir: result_dir(ctx)?,
            drawio_identity: drawio_identity(&drawio_bin),
            drawio_bin,
            drawio_args: get_preprocessor_value(ctx, "drawio-args", Vec::new()),
            jobs: get_preprocessor_value(ctx, "jobs", default_jobs).max(1),
        })
    }
}

/// The directory in the book's `src` directory that contains the SVGs.
/// Can be set via [preprocessor.drawio.result-dir] in book.toml.
///
/// mdBook only copies files inside `src` into the book, so the directory may
/// not be absolute or leave `src`.
fn result_dir(ctx: &PreprocessorContext) -> Result<PathBuf, Error> {
    let dir = get_preprocessor_value(ctx, "result-dir", "mdbook-drawio".to_string());
    let path = Path::new(&dir);
    if path.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir)) {
        Ok(path.components().collect())
    } else {
        Err(Error::msg(format!(
            "`preprocessor.drawio.result-dir` is `{dir}`, but it must be a relative path inside \
             the book's `src` directory, because mdBook only copies files from there into the book"
        )))
    }
}

fn get_preprocessor_value<T: serde::de::DeserializeOwned>(
    ctx: &PreprocessorContext,
    key: &str,
    default: T,
) -> T {
    let config_key = format!("preprocessor.drawio.{key}");
    match ctx.config.get::<T>(&config_key) {
        Ok(Some(value)) => value,
        Ok(None) => default,
        Err(err) => {
            error!("Failed to read `{config_key}` from book config: {err}");
            default
        }
    }
}

/// One SVG to export: a page of a diagram.
struct Export {
    /// Path of the diagram as written in the directive, for messages.
    name: String,
    /// Absolute path of the diagram.
    source: PathBuf,
    page: u32,
    /// For a step of a step-by-step diagram: the step, and the one-page
    /// diagram to export instead of `source`.
    step: Option<(usize, String)>,
    /// Absolute path of the SVG.
    svg: PathBuf,
    /// Hash of everything that affects the SVG, see [`stamp`].
    stamp: String,
    /// Whether the SVG is missing or has another stamp.
    stale: bool,
}

/// Replaces the directives in a chapter with image links, and adds the SVGs
/// they need to `exports`.
fn process_chapter(
    ctx: &PreprocessorContext,
    config: &Config,
    ch: &mut Chapter,
    exports: &mut BTreeMap<PathBuf, Export>,
) -> Result<(), String> {
    let directives = find_directives(&ch.content)?;
    if directives.is_empty() {
        return Ok(());
    }

    let mut content = String::with_capacity(ch.content.len());
    let mut last = 0;
    let mut has_steps = false;
    for directive in &directives {
        debug!("Processing directive: {directive:?}");
        let source = ctx.root.join(&directive.path).components().collect::<PathBuf>();
        let bytes = std::fs::read(&source)
            .map_err(|e| format!("cannot read {}: {e}", directive.path))?;
        let in_diagram = |e| format!("{}: {e}", directive.path);

        let alt = match &directive.alt {
            Some(alt) => alt.clone(),
            None => {
                let stem = Path::new(&directive.path).file_stem().unwrap().to_string_lossy();
                format!("Diagram {stem} page {}", directive.page + 1)
            }
        };
        let link = |svg: &Path| link_from_chapter(ch.path.as_deref(), &config.result_dir.join(svg));
        let replacement = match directive.steps {
            None => {
                let svg = svg_path(&directive.path, directive.page, None);
                add_export(config, exports, directive, &source, &bytes, &svg, None)?;
                image_markdown(&alt, &link(&svg))
            }
            Some(Steps::Layers) => {
                has_steps = true;
                let layers = steps::Layers::read(&bytes, directive.page).map_err(in_diagram)?;
                let mut links = Vec::new();
                for step in 0..layers.names.len() {
                    let svg = svg_path(&directive.path, directive.page, Some(step));
                    let diagram = (step, layers.step_diagram(step));
                    add_export(config, exports, directive, &source, &bytes, &svg, Some(diagram))?;
                    links.push(link(&svg));
                }
                steps::figure_html(&alt, &links, &layers.names)
            }
        };
        content.push_str(&ch.content[last..directive.range.start]);
        content.push_str(&replacement);
        last = directive.range.end;
    }
    content.push_str(&ch.content[last..]);
    if has_steps {
        content.push_str(steps::ASSETS);
    }
    ch.content = content;
    Ok(())
}

/// Adds the export of `svg` (relative to the result directory), unless
/// another directive added it already.
fn add_export(
    config: &Config,
    exports: &mut BTreeMap<PathBuf, Export>,
    directive: &Directive,
    source: &Path,
    bytes: &[u8],
    svg: &Path,
    step: Option<(usize, String)>,
) -> Result<(), String> {
    let svg_abs = config.src.join(&config.result_dir).join(svg);
    if let Some(export) = exports.get(&svg_abs) {
        if export.source != source {
            return Err(format!(
                "{} and {} would both be exported to {}",
                export.name,
                directive.path,
                config.result_dir.join(svg).display()
            ));
        }
        return Ok(());
    }
    let stamp = stamp(config, bytes, directive.page, step.as_ref().map(|(n, _)| *n));
    let stale = read_stamp(&svg_abs).as_deref() != Some(stamp.as_str());
    debug!("{svg_abs:?} is {}", if stale { "stale" } else { "up to date" });
    if stale && step.is_none() {
        validate_diagram(bytes, source, directive.page)
            .map_err(|e| format!("{}: {e}", directive.path))?;
    }
    let export = Export {
        name: directive.path.clone(),
        source: source.to_path_buf(),
        page: directive.page,
        step,
        svg: svg_abs.clone(),
        stamp,
        stale,
    };
    exports.insert(svg_abs, export);
    Ok(())
}

/// The path of the SVG for a page of the diagram at `source` (relative to the
/// book root), relative to the result directory.
///
/// The SVG keeps the directory of the source, so that diagrams with the same
/// file name in different directories do not overwrite each other. A source
/// outside the book root gets a directory named after a hash of its path.
pub fn svg_path(source: &str, page: u32, step: Option<usize>) -> PathBuf {
    let source = Path::new(source);
    let stem = source.file_stem().unwrap().to_string_lossy();
    let file = match step {
        None => format!("{stem}-page-{page}.svg"),
        Some(step) => format!("{stem}-page-{page}-step-{step}.svg"),
    };
    let parent = source.parent().unwrap_or(Path::new(""));
    let mut dir = PathBuf::new();
    for component in parent.components() {
        match component {
            Component::Normal(c) => dir.push(c),
            Component::CurDir => {}
            _ => {
                let hash = hex(&Sha256::digest(parent.to_string_lossy().as_bytes()));
                return Path::new("_external").join(&hash[..12]).join(file);
            }
        }
    }
    dir.join(file)
}

/// The link from a chapter to `target`, a path relative to `src`.
pub fn link_from_chapter(chapter: Option<&Path>, target: &Path) -> String {
    let depth = chapter
        .and_then(|p| p.parent())
        .map_or(0, |p| p.components().filter(|c| matches!(c, Component::Normal(_))).count());
    let mut parts = vec!["..".to_string(); depth];
    parts.extend(target.components().map(|c| c.as_os_str().to_string_lossy().into_owned()));
    parts.join("/")
}

/// A Markdown image. The link is in angle brackets so that it may contain spaces.
fn image_markdown(alt: &str, link: &str) -> String {
    let mut escaped = String::with_capacity(alt.len());
    for c in alt.chars() {
        if matches!(c, '\\' | '[' | ']') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    format!("![{escaped}](<{link}>)")
}

/// Checks that a diagram is well-formed XML and has the page, because draw.io
/// exports a damaged file without an error.
fn validate_diagram(bytes: &[u8], path: &Path, page: u32) -> Result<(), String> {
    // draw.io can also read diagrams embedded in PNG files, which we cannot check.
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !["drawio", "dio", "xml", "svg"].contains(&ext.to_ascii_lowercase().as_str()) {
        return Ok(());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| format!("not valid UTF-8: {e}"))?;
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("not a valid diagram: {e}"))?;
    let root = doc.root_element();
    let pages = match root.tag_name().name() {
        "mxfile" => root.children().filter(|n| n.has_tag_name("diagram")).count(),
        "mxGraphModel" => 1,
        _ => return Ok(()),
    };
    if page as usize >= pages {
        return Err(format!("has no page {page} (it has {pages}, numbered from 0)"));
    }
    Ok(())
}

/// Identifies the draw.io installation, so that a new version exports the
/// diagrams again. Running `drawio --version` takes as long as an export, so
/// this uses the path, size and modification time of the binary instead.
fn drawio_identity(bin: &str) -> String {
    let Ok(path) = which::which(bin).map(|p| p.canonicalize().unwrap_or(p)) else {
        return bin.to_string();
    };
    let (len, mtime) = std::fs::metadata(&path).map_or((0, 0), |m| {
        let mtime = m.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok());
        (m.len(), mtime.map_or(0, |d| d.as_secs()))
    });
    format!("{} {len} {mtime}", path.display())
}

/// A hash of everything that affects the SVG of a page: the diagram, the
/// page and step, the draw.io arguments and the draw.io installation.
fn stamp(config: &Config, source: &[u8], page: u32, step: Option<usize>) -> String {
    let mut hasher = Sha256::new();
    for part in [b"mdbook-drawio stamp 1".as_slice(), config.drawio_identity.as_bytes()] {
        hasher.update(part);
        hasher.update([0]);
    }
    for arg in &config.drawio_args {
        hasher.update(arg.as_bytes());
        hasher.update([0]);
    }
    hasher.update(page.to_le_bytes());
    match step {
        None => hasher.update([0]),
        Some(step) => {
            hasher.update([1]);
            hasher.update((step as u64).to_le_bytes());
        }
    }
    hasher.update(source);
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The stamp is stored as a comment at the end of the SVG.
const STAMP_PREFIX: &str = "<!-- mdbook-drawio-stamp: ";
const STAMP_SUFFIX: &str = " -->";

fn read_stamp(svg: &Path) -> Option<String> {
    let content = std::fs::read_to_string(svg).ok()?;
    let start = content.rfind(STAMP_PREFIX)? + STAMP_PREFIX.len();
    let len = content[start..].find(STAMP_SUFFIX)?;
    Some(content[start..start + len].to_string())
}

fn write_stamp(svg: &Path, stamp: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(svg)?;
    writeln!(file, "\n{STAMP_PREFIX}{stamp}{STAMP_SUFFIX}")
}

/// Exports the SVGs, running up to `config.jobs` draw.io processes at once.
fn export_all(config: &Config, exports: &[Export]) -> Result<(), Error> {
    if exports.is_empty() {
        return Ok(());
    }
    if which::which(&config.drawio_bin).is_err() {
        return Err(Error::msg(format!(
            "draw.io (`{}`) was not found. Install it, or set `preprocessor.drawio.drawio-bin`",
            config.drawio_bin
        )));
    }
    let start = Instant::now();
    let next = AtomicUsize::new(0);
    let errors = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..config.jobs.min(exports.len()) {
            scope.spawn(|| {
                while let Some(export) = exports.get(next.fetch_add(1, Ordering::Relaxed)) {
                    if let Err(e) = drawio_export(config, export) {
                        errors.lock().unwrap().push(e.to_string());
                    }
                }
            });
        }
    });
    let errors = errors.into_inner().unwrap();
    if !errors.is_empty() {
        return Err(Error::msg(errors.join("\n")));
    }
    info!("Exported {} diagram page(s) in {:.1?}", exports.len(), start.elapsed());
    Ok(())
}

/// Invokes drawio to export a diagram to SVG format.
fn drawio_export(config: &Config, export: &Export) -> Result<(), Error> {
    let Export { name, source, page, step, svg, stamp, .. } = export;
    // The mdBook directive uses zero-based pages, while draw.io's CLI is one-based.
    let mut cli_page = (page + 1).to_string();
    let mut input = source.clone();
    let _step_file = match step {
        None => {
            info!("Exporting {name} page {page}");
            None
        }
        Some((step, diagram)) => {
            info!("Exporting {name} page {page} step {step}");
            let file = TempFile::new(svg.with_extension("drawio"), diagram)?;
            input = file.0.clone();
            cli_page = "1".to_string();
            Some(file)
        }
    };

    if let Some(dir) = svg.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let mut cmd = Command::new(&config.drawio_bin);
    cmd.env("ELECTRON_DISABLE_GPU", "1")
        // The Nix drawio wrapper writes to an xvfb frame buffer so wayland does
        // not work
        .env_remove("NIXOS_OZONE_WL")
        // Some tools (e.g. Electron-based editors) set this for their child
        // processes, which makes draw.io start as plain Node.js and fail.
        .env_remove("ELECTRON_RUN_AS_NODE")
        .arg("--export")
        .arg("--format")
        .arg("svg")
        .arg("--page-index")
        .arg(&cli_page)
        .arg("--output")
        .arg(svg)
        .args(&config.drawio_args)
        .arg(&input);

    debug!("Full command: {cmd:?}");

    let output = cmd.output().map_err(|e| {
        Error::msg(format!("Failed to execute drawio command `{}`: {e}", config.drawio_bin))
    })?;
    debug!("Command exit status: {status:?}", status = output.status);
    debug!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    debug!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        let code = output.status.code().map_or("unknown".to_string(), |c| c.to_string());
        return Err(Error::msg(format!(
            "drawio exited with status {code} for {name}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    if !svg.exists() {
        return Err(Error::msg(format!("drawio did not create {}", svg.display())));
    }
    write_stamp(svg, stamp)?;
    Ok(())
}

/// A file that is removed when dropped.
struct TempFile(PathBuf);

impl TempFile {
    fn new(path: PathBuf, content: &str) -> std::io::Result<TempFile> {
        std::fs::write(&path, content)?;
        Ok(TempFile(path))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_path_keeps_directory() {
        assert_eq!(svg_path("diagrams/a/overview.drawio", 1, None), Path::new("diagrams/a/overview-page-1.svg"));
        assert_eq!(svg_path("./x.drawio", 0, Some(2)), Path::new("x-page-0-step-2.svg"));
        assert_ne!(svg_path("diagrams/a/o.drawio", 0, None), svg_path("diagrams/b/o.drawio", 0, None));
    }

    #[test]
    fn svg_path_outside_book_root_stays_in_result_dir() {
        let a = svg_path("../shared/o.drawio", 0, None);
        let b = svg_path("../other/o.drawio", 0, None);
        assert!(a.starts_with("_external") && b.starts_with("_external"));
        assert_ne!(a, b);
        assert!(a.components().all(|c| matches!(c, Component::Normal(_))));
    }

    #[test]
    fn link_from_nested_chapter() {
        let target = Path::new("gen/d/o-page-0.svg");
        assert_eq!(link_from_chapter(Some(Path::new("intro.md")), target), "gen/d/o-page-0.svg");
        assert_eq!(link_from_chapter(Some(Path::new("a/b/c.md")), target), "../../gen/d/o-page-0.svg");
        assert_eq!(link_from_chapter(None, target), "gen/d/o-page-0.svg");
    }

    #[test]
    fn image_markdown_escapes_alt() {
        assert_eq!(image_markdown(r"a [b] \c", "x y.svg"), r"![a \[b\] \\c](<x y.svg>)");
    }

    #[test]
    fn validates_diagrams() {
        let path = Path::new("d.drawio");
        let two_pages = br#"<mxfile><diagram id="a"/><diagram id="b"/></mxfile>"#;
        assert!(validate_diagram(two_pages, path, 1).is_ok());
        assert!(validate_diagram(two_pages, path, 2).is_err());
        assert!(validate_diagram(br#"<mxfile><diagram id="a"><mxGraphModel>"#, path, 0).is_err());
        assert!(validate_diagram(b"not xml", Path::new("d.png"), 0).is_ok());
    }

    #[test]
    fn stamp_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let svg = dir.path().join("a.svg");
        std::fs::write(&svg, "<svg/>").unwrap();
        assert_eq!(read_stamp(&svg), None);
        write_stamp(&svg, "abc").unwrap();
        assert_eq!(read_stamp(&svg).as_deref(), Some("abc"));
    }

    #[test]
    fn stamp_depends_on_args_and_drawio() {
        let mut config = Config {
            src: PathBuf::new(),
            result_dir: PathBuf::new(),
            drawio_bin: "drawio".into(),
            drawio_args: vec![],
            jobs: 1,
            drawio_identity: "drawio 1".into(),
        };
        let a = stamp(&config, b"<mxfile/>", 0, None);
        assert_ne!(a, stamp(&config, b"<mxfile/>", 1, None));
        assert_ne!(a, stamp(&config, b"<mxfile/>", 0, Some(0)));
        config.drawio_args = vec!["--svg-theme".into(), "light".into()];
        let b = stamp(&config, b"<mxfile/>", 0, None);
        assert_ne!(a, b);
        config.drawio_identity = "drawio 2".into();
        assert_ne!(b, stamp(&config, b"<mxfile/>", 0, None));
    }
}
