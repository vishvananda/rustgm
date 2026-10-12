use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
struct Files;
impl Sources for Files {
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        Ok(Some(InputFile {
            identity: fs::canonicalize(path).map_err(|e| e.to_string())?,
            source: SourceFile::new(bytes)?,
        }))
    }
}
pub fn names(
    source: SourceFile,
    path: Option<&str>,
    config: &Config,
    count: bool,
    out: &mut impl Write,
) -> Result<(), String> {
    let identity = path.filter(|p| *p != "-").map_or_else(
        || Ok(PathBuf::from("stdin.rs")),
        |p| fs::canonicalize(p).map_err(|e| e.to_string()),
    )?;
    let root = InputFile { identity, source };
    let mut package = resolve::load::load(root, &mut Files, config).map_err(|e| diagnostic(&e))?;
    let facts = resolve::analyze(&mut package).map_err(|e| diagnostic(&e))?;
    if count {
        writeln!(
            out,
            "files {} modules {} names {} scopes {} definitions {} references {}",
            package.units.len(),
            package.modules.len(),
            package.names.len(),
            facts.scopes.len(),
            facts.definitions.len(),
            facts.references.len()
        )
        .map_err(|e| e.to_string())
    } else {
        resolve::render::write(&package, &facts, out).map_err(|e| e.to_string())
    }
}
fn diagnostic(error: &resolve::package::Error) -> String {
    format!(
        "file {}: {}: {}",
        error.file.0, error.span.start, error.message
    )
}
