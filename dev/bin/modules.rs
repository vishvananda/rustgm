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
    fn resource(&mut self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    fn environment(&mut self, name: &str) -> Result<Option<String>, String> {
        match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(std::env::VarError::NotUnicode(_)) => {
                Err(format!("environment variable {name:?} is not Unicode"))
            }
        }
    }

    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        Ok(Some(InputFile {
            identity: fs::canonicalize(path).map_err(|e| e.to_string())?,
            path: path.to_owned(),
            source: SourceFile::new(bytes)?,
        }))
    }
}
#[derive(Clone, Copy)]
pub enum Mode {
    Names,
    Expanded,
}
pub fn run(
    source: SourceFile,
    path: Option<&str>,
    config: &Config,
    crate_name: Option<&str>,
    count: bool,
    mode: Mode,
    out: &mut impl Write,
) -> Result<(), String> {
    let identity = path.filter(|p| *p != "-").map_or_else(
        || Ok(PathBuf::from("stdin.rs")),
        |p| fs::canonicalize(p).map_err(|e| e.to_string()),
    )?;
    let root = InputFile {
        identity,
        path: path.filter(|p| *p != "-").unwrap_or("<anon>").into(),
        source,
    };
    if matches!(mode, Mode::Expanded) {
        let report =
            resolve::expansion::analyze_with_crate_name(root, &mut Files, config, crate_name)
                .and_then(resolve::analyze_expanded)
                .map_err(|e| e.to_string())?;
        return if count {
            resolve::render::count_expanded(&report, out)
        } else {
            resolve::render::expanded(&report, out)
        }
        .map_err(|e| e.to_string());
    }
    let mut package = resolve::load::load(root, &mut Files, config).map_err(|e| e.to_string())?;
    let facts = resolve::analyze(&mut package).map_err(|e| e.to_string())?;
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
