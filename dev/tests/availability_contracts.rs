//! Independently frozen local policies enter the ordinary semantic pipeline.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::path::{Path, PathBuf};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CASES: &[(&str, bool)] = &[
    ("private-function", true),
    ("hidden-public-function", true),
    ("reexport-missing-target", false),
    ("reexport-marked-target", false),
    ("stable-module-public-child", false),
    ("stable-module-private-child", true),
    ("unstable-module-public-child", true),
    ("stable-child-overrides-unstable-parent", false),
    ("unstable-root-inherited", true),
    ("missing-root-annotation", false),
    ("duplicate-stable", false),
    ("duplicate-feature-key", false),
    ("stable-minor-version", true),
    ("stable-current-version", true),
    ("stable-prerelease-version", false),
    ("invalid-feature-name", false),
    ("zero-issue", false),
    ("missing-issue", false),
    ("nonzero-issue", true),
    ("signature-exposes-unmarked-type", false),
    ("signature-exposes-marked-type", false),
    ("shared-file-both-unstable", true),
    ("shared-file-stable-second", false),
    ("shared-file-private-second", true),
    ("loaded-inner-unstable", true),
    ("outer-inner-duplicate", false),
    ("local-unstable-runtime", false),
    ("stable-override-runtime", false),
    ("local-unstable-check", true),
    ("stable-override-check", true),
    ("marked-import-and-target", true),
    ("marked-import-unmarked-target", false),
    ("public-type-through-signature", false),
    ("marked-public-type-through-signature", false),
    ("marked-reexported-public-type", false),
    ("unmarked-group-import", false),
    ("marked-group-import", true),
    ("marked-glob-import", true),
];

struct Files;
impl Sources for Files {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        Ok(Some(InputFile {
            identity: path.into(),
            path: path.into(),
            source: SourceFile::new(bytes)?,
        }))
    }
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/library-stability/owners")
        .join(name)
        .join("lib.rs")
}
fn analyze(path: &Path) -> Result<resolve::expansion::Report> {
    let input = Files.read(path)?.ok_or("fixture")?;
    let expanded = resolve::expansion::analyze(input, &mut Files, &Config::default())?;
    Ok(resolve::analyze_expanded(expanded)?)
}

#[test]
fn local_owner_and_import_outcomes_preserve_independent_observations() -> Result {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        match analyze(&fixture(name)) {
            Ok(report) if accepted => {
                rustgm::pipeline::analyze_source(&report)?;
            }
            Ok(_) => failures.push(format!("{name}: unexpected acceptance")),
            Err(error) if accepted => failures.push(format!("{name}: {error}")),
            Err(error) if !expected_diagnostic(name, &error.to_string()) => {
                failures.push(format!("{name}: unrelated rejection: {error}"));
            }
            Err(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn expected_diagnostic(name: &str, message: &str) -> bool {
    let expected = match name {
        "local-unstable-runtime" | "stable-override-runtime" => "unresolved macro",
        "signature-exposes-unmarked-type"
        | "signature-exposes-marked-type"
        | "public-type-through-signature"
        | "marked-public-type-through-signature"
        | "marked-reexported-public-type" => "local library stability profile",
        "missing-root-annotation" => "crate has missing library stability",
        "duplicate-stable" => "duplicate or conflicting library stability",
        "duplicate-feature-key" => "duplicate library stability argument",
        "stable-prerelease-version" => "invalid stable library version",
        "invalid-feature-name" => "library feature must be an identifier",
        "zero-issue" => "invalid unstable library issue",
        "missing-issue" => "unstable library policy requires issue",
        "outer-inner-duplicate" => "duplicate outer and inner library stability",
        "reexport-marked-target" | "unmarked-group-import" => {
            "public import has missing library stability"
        }
        _ => "public declaration has missing library stability",
    };
    message.contains(expected)
}

#[test]
fn deferred_features_use_enabled_expanded_definitions_and_keep_const_separate() -> Result {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../pa31/tests/spec/library-stability/api");
    for (name, accepted) in [
        ("defined-after-request", true),
        ("generated-definition", true),
        ("local-unstable-use", true),
        ("inherited-unstable", true),
        ("undefined-request", false),
        ("disabled-definition", false),
        ("conflicting-stability", false),
        ("missing-staged-api", false),
        ("missing-const-stability", false),
        ("runtime-unstable-const", false),
    ] {
        let result = analyze(&root.join(format!("{name}.rs")));
        if accepted {
            rustgm::pipeline::analyze_source(&result?)?;
        } else {
            let Err(error) = result else {
                return Err(format!("{name}: unexpected acceptance").into());
            };
            let expected = match name {
                "undefined-request" | "disabled-definition" => {
                    "undefined library feature request"
                }
                "conflicting-stability" => "duplicate or conflicting library stability",
                "missing-staged-api" => "library stability requires staged_api",
                // These valid/invalid const hypotheses remain outside this
                // candidate profile; an unsupported gate is not Rust evidence.
                "missing-const-stability" => "local library stability profile",
                "runtime-unstable-const" => {
                    "const, default-body and macro library stability contracts are not implemented"
                }
                _ => return Err("undeclared API rejection diagnostic".into()),
            };
            assert!(error.to_string().contains(expected), "{name}: {error}");
        }
    }
    Ok(())
}

#[test]
fn completed_local_policy_cannot_be_dropped_by_metadata_export() -> Result {
    let report = analyze(&fixture("marked-import-and-target"))?;
    let mut analysis = rustgm::pipeline::analyze_source(&report)?;
    let identity = rustgm::support::crates::CrateIdentity::new("local", "stability")?;
    let result = rustgm::sema::crates::export(&report, &mut analysis.facts, identity);
    let Err(error) = result else {
        return Err("stability was silently omitted from metadata".into());
    };
    assert!(
        error
            .to_string()
            .contains("library stability metadata export")
    );
    Ok(())
}

#[test]
#[ignore = "requires CPPGM backend; exact sources have independent native observations"]
fn local_unstable_calls_and_stable_overrides_execute() -> Result {
    use std::process::Command;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root.join("obj/parallel/pa31-library-native").join(format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder)?;
    for name in ["local-unstable-check", "stable-override-check"] {
        let source = std::fs::read_to_string(fixture(name))?;
        let input = folder.join(format!("{name}.lowir"));
        let binary = folder.join(format!("{name}.native"));
        std::fs::write(
            &input,
            crate::edition_packages::lowir(&source, rustgm::support::edition::Edition::Rust2024)?,
        )?;
        crate::edition_packages::run(
            &folder,
            &format!("{name}-backend"),
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg(input)
                .arg("-o")
                .arg(&binary),
        )?;
        crate::edition_packages::run(&folder, name, &mut Command::new(binary))?;
        for stream in ["stdout", "stderr"] {
            assert!(std::fs::read(folder.join(format!("{name}.{stream}")))?.is_empty());
        }
    }
    Ok(())
}
