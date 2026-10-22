use rustgm::support::hash::HashMap;
use rustgm::{
    expand::config::Config,
    resolve::load::{self, InputFile, Sources},
    support::source::SourceFile,
};
use std::path::{Path, PathBuf};
#[derive(Default)]
struct Memory {
    files: HashMap<PathBuf, String>,
    reads: Vec<PathBuf>,
}
impl Sources for Memory {
    fn read(&mut self, path: &Path) -> Result<Option<InputFile>, String> {
        self.reads.push(path.into());
        self.files
            .get(path)
            .map(|text| input(path, text))
            .transpose()
    }
}
fn input(path: &Path, text: &str) -> Result<InputFile, String> {
    Ok(InputFile {
        identity: path.into(),
        path: path.into(),
        source: SourceFile::new(text.as_bytes().to_vec())?,
    })
}
fn memory(files: &[(&str, &str)]) -> Memory {
    Memory {
        files: files
            .iter()
            .map(|(path, text)| (PathBuf::from(path), (*text).into()))
            .collect(),
        reads: Vec::new(),
    }
}
#[test]
fn ordinary_and_explicit_module_paths_have_distinct_directory_ownership() -> Result<(), String> {
    let root = input(
        Path::new("src/lib.rs"),
        "mod a; #[path=\"other.rs\"] mod b;",
    )?;
    let mut sources = memory(&[
        ("src/a.rs", "mod child; mod inline {mod leaf;}"),
        ("src/a/child.rs", ""),
        ("src/a/inline/leaf.rs", ""),
        ("src/other.rs", "mod child;"),
        ("src/child.rs", ""),
    ]);
    let package =
        load::load(root, &mut sources, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(package.units.len(), 6);
    assert_eq!(package.modules.len(), 7);
    assert!(sources.reads.contains(&PathBuf::from("src/child.rs")));
    assert!(
        sources
            .reads
            .contains(&PathBuf::from("src/a/inline/leaf.rs"))
    );
    Ok(())
}
#[test]
fn disabled_modules_never_read_files_and_cfg_attr_supplies_path() -> Result<(), String> {
    let root = input(
        Path::new("lib.rs"),
        "#[cfg(false)] mod absent; #[cfg_attr(true,path=\"yes.rs\")] mod selected;",
    )?;
    let mut sources = memory(&[("yes.rs", "")]);
    let package =
        load::load(root, &mut sources, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(package.modules.len(), 2);
    assert_eq!(sources.reads, vec![PathBuf::from("yes.rs")]);
    Ok(())
}
#[test]
fn shared_physical_files_are_parsed_once_but_have_distinct_modules() -> Result<(), String> {
    let root = input(
        Path::new("lib.rs"),
        "#[path=\"same.rs\"] mod a; #[path=\"same.rs\"] mod b;",
    )?;
    let mut sources = memory(&[("same.rs", "fn shared(){}")]);
    let package =
        load::load(root, &mut sources, &Config::default()).map_err(|e| format!("{e:?}"))?;
    assert_eq!(package.units.len(), 2);
    assert_eq!(package.modules.len(), 3);
    assert_eq!(package.modules[1].file, package.modules[2].file);
    Ok(())
}
#[test]
fn ambiguous_missing_recursive_and_block_modules_are_diagnosed() -> Result<(), String> {
    for (text, files, diagnostic) in [
        ("mod absent;", vec![], "not found"),
        ("mod a;", vec![("a.rs", ""), ("a/mod.rs", "")], "both"),
        (
            "#[path=\"lib.rs\"] mod a;",
            vec![("lib.rs", "")],
            "circular",
        ),
        ("fn f(){mod a;}", vec![("a.rs", "")], "block"),
        ("mod é;", vec![("é.rs", "")], "non-ASCII"),
        ("#[path=3] mod a;", vec![], "string literal"),
    ] {
        let root = input(Path::new("lib.rs"), text)?;
        let mut sources = memory(&files);
        let Err(error) = load::load(root, &mut sources, &Config::default()) else {
            return Err(format!("accepted {text}"));
        };
        assert!(error.message.contains(diagnostic), "{error:?}");
    }
    Ok(())
}
