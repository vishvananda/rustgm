//! Absolute source paths reuse edition-aware lookup and actual macro declarations.
use rustgm::{
    expand::config::Config,
    resolve::{
        bindings::Context,
        expansion,
        load::{InputFile, Sources},
    },
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency},
        source::SourceFile,
    },
};
use std::{
    path::Path,
    process::{Command, Output},
    sync::Arc,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "../pa34/tests/spec/absolute-source-macros";
const IDENTITY: &str = "absolute_source_paths_bringup";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

fn run(name: &str, app: &str, edition: &str, profile: &str, guests: &str) -> Result<Output> {
    let (folder, _) = crate::guest_crates::prepare(name, &[])?;
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command.args([
        "--emit-expanded-crate",
        "--cap-lints=allow",
        "--edition=2024",
    ]);
    if profile == "ordinary" {
        return Ok(command.arg(inputs.join(app)).output()?);
    }
    std::fs::copy(inputs.join("host.rs"), folder.join("core.rs"))?;
    let skip = folder.join("skip.tsv");
    std::fs::write(&skip, "")?;
    command.args(["--crate-name", "core", "--crate-disambiguator", IDENTITY]);
    command.args(["--source-core", &format!("core@{IDENTITY}")]);
    command.arg("--bringup-skip-list").arg(skip);
    for (index, specification) in guests.split('|').enumerate() {
        let (guest, file) = specification
            .split_once('=')
            .ok_or("invalid guest contract")?;
        let path = folder.join(format!("guest-{index}.rs"));
        std::fs::copy(inputs.join(file), &path)?;
        command
            .arg("--bringup-guest")
            .arg(format!("{guest}={}", path.display()));
    }
    assert!(["2015", "2021", "2024"].contains(&edition));
    Ok(command.arg(folder.join("core.rs")).output()?)
}

#[test]
fn actual_source_macro_roots_and_reexports_preserve_absolute_path_contracts() -> Result {
    let cases = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(PACKET)
            .join("cases.tsv"),
    )?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, app, edition, expected, profile, guests] = columns.as_slice() else {
            return Err("malformed absolute macro contract".into());
        };
        let output = run(name, app, edition, profile, guests)?;
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(expected.parse()?),
            "{name}: {diagnostic}"
        );
        if *expected == "1" {
            assert!(output.stdout.is_empty(), "{name}");
            if *profile == "ordinary" {
                assert!(
                    diagnostic.contains("external macro paths require external metadata"),
                    "{diagnostic}"
                );
            }
        }
        count += 1;
    }
    assert_eq!(count, 15);
    Ok(())
}

#[test]
fn source_mode_does_not_convert_imported_macro_rows_into_source_declarations() -> Result {
    let (folder, _) = crate::guest_crates::prepare("absolute-imported-boundary", &[])?;
    let inputs = Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET);
    let metadata = folder.join("base.rgcrate");
    let provider = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args(["--emit-lowir", "--crate-type=lib", "--crate-name", "base"])
        .args(["--crate-disambiguator", "absolute_macro_boundary"])
        .arg("--metadata-output")
        .arg(&metadata)
        .arg("-o")
        .arg(folder.join("base.lowir"))
        .arg(inputs.join("metadata-provider.rs"))
        .output()?;
    assert_eq!(
        provider.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&provider.stderr)
    );
    let artifact = crates::decode(&std::fs::read(&metadata)?)?;
    let alias = Dependency::new("base".into(), artifact.identity().clone())?;
    let current = CrateIdentity::new("core", IDENTITY)?;
    let context = Context {
        crate_name: Some("core".into()),
        current: Some(current.clone()),
        dependencies: Some(Arc::new(Dependencies::new(vec![artifact], vec![alias])?)),
        mode: Mode::Source { provider: current },
        ..Default::default()
    };
    let host = std::fs::read_to_string(inputs.join("host.rs"))?;
    for (name, expected) in [("metadata-relative.rs", 0), ("metadata-absolute.rs", 1)] {
        let text = format!("{host}\n{}", std::fs::read_to_string(inputs.join(name))?);
        let input = InputFile {
            identity: name.into(),
            path: folder.join(name),
            source: SourceFile::new(text.into_bytes())?,
        };
        let result =
            expansion::analyze_with_context(input, &mut NoFiles, &Config::default(), &context);
        if expected == 0 {
            let report = result.map_err(|error| format!("{name}: {error}"))?;
            assert!(report.package.source_core().is_some());
        } else {
            let diagnostic = result
                .err()
                .ok_or("absolute metadata macro was expanded")?
                .to_string();
            assert!(
                diagnostic.contains("external macro paths require external metadata"),
                "{diagnostic}"
            );
        }
    }
    Ok(())
}
