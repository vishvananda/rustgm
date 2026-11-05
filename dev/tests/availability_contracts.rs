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
    ("signature-exposes-unmarked-type", true),
    ("signature-exposes-marked-type", true),
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
    ("marked-public-type-through-signature", true),
    ("marked-reexported-public-type", true),
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
fn local_owner_and_import_outcomes_preserve_independent_observations() {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let checked = analyze(&fixture(name)).and_then(|report| {
            rustgm::pipeline::analyze_source(&report)?;
            Ok(report)
        });
        match checked {
            Ok(_) if accepted => {}
            Ok(_) => failures.push(format!("{name}: unexpected acceptance")),
            Err(error) if accepted => failures.push(format!("{name}: {error}")),
            Err(error) if !expected_diagnostic(name, &error.to_string()) => {
                failures.push(format!("{name}: unrelated rejection: {error}"));
            }
            Err(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn expected_diagnostic(name: &str, message: &str) -> bool {
    let expected = match name {
        "local-unstable-runtime" | "stable-override-runtime" => "unresolved macro",
        "public-type-through-signature" => "missing library stability",
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
        ("runtime-unstable-const", true),
    ] {
        let result = analyze(&root.join(format!("{name}.rs")));
        if accepted {
            rustgm::pipeline::analyze_source(&result?)?;
        } else {
            let Err(error) = result else {
                return Err(format!("{name}: unexpected acceptance").into());
            };
            let expected = match name {
                "undefined-request" | "disabled-definition" => "undefined library feature request",
                "conflicting-stability" => "duplicate or conflicting library stability",
                "missing-staged-api" => "library stability requires staged_api",
                "missing-const-stability" => "missing const library stability attribute",
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

#[test]
fn const_policy_targets_and_call_permissions_match_the_actual_owners() -> Result {
    let rows = crate::resolve::fixture("pa31/tests/spec/library-stability/const-owners/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for row in rows.lines() {
        let mut fields = row.split('\t');
        let name = fields.next().ok_or("case name")?;
        let path = fields.next().ok_or("source path")?;
        let status = fields.next().ok_or("expected status")?;
        assert!(matches!(status, "0" | "1") && fields.next().is_none());
        let outcome = crate::provider_traits::lower(&crate::resolve::fixture(path)?);
        if outcome.is_ok() != (status == "0") {
            failures.push(format!("{name}: {:?}", outcome.err()));
        } else if let Err(error) = outcome {
            assert!(
                !error
                    .to_string()
                    .contains("semantic contract is not implemented"),
                "{name}: {error}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 25);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn inherited_const_policy_never_changes_the_actual_callable_constness() -> Result {
    use rustgm::resolve::stability::Library;
    for (name, constant_container) in [
        ("ordinary-trait", false),
        ("ordinary-inherent", false),
        ("ordinary-impl", false),
        ("const-trait", true),
    ] {
        let source = crate::resolve::fixture(&format!(
            "pa31/tests/spec/library-stability/const-owners/{name}.rs"
        ))?;
        let report = crate::provider_traits::report(&source)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let function = analysis
            .facts
            .functions
            .iter()
            .find(|function| {
                function.body.is_some()
                    && report
                        .package
                        .names
                        .get(report.resolution.definitions[function.definition.0 as usize].name)
                        == "read"
            })
            .ok_or("actual provided function")?;
        assert!(!analysis.facts.is_const_function(function.definition));
        let owner = analysis
            .facts
            .associated
            .owner(function.definition)
            .and_then(|id| analysis.facts.associated.get(id))
            .ok_or("actual callable owner")?;
        assert_eq!(
            matches!(
                owner.kind,
                rustgm::sema::associated::OwnerKind::Trait { constant: true, .. }
                    | rustgm::sema::associated::OwnerKind::Impl { constant: true, .. }
            ),
            constant_container
        );
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("actual policy inventory")?;
        assert_eq!(
            inventory
                .const_declaration(function.definition)
                .ok_or("inherited policy")?
                .feature
                .as_ref(),
            "marker_const"
        );
        assert_eq!(size_of_val(inventory), 120);
        assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn ordinary_annotated_trait_and_impl_bodies_execute_without_const_permission() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/const-policy-owners-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for name in ["ordinary-trait", "ordinary-inherent", "ordinary-impl"] {
        let source = crate::resolve::fixture(&format!(
            "pa31/tests/spec/library-stability/const-owners/{name}.rs"
        ))?;
        crate::ffi_program::check(
            root,
            &folder,
            name,
            &crate::provider_traits::lower(&source)?,
            &crate::resolve::fixture("pa31/tests/spec/library-stability/const-owners/observer.c")?,
            "7\n",
        )?;
    }
    let original =
        crate::resolve::fixture("pa31/tests/spec/source-assign-ten/ordinary-retained-policy.rs")?;
    crate::ffi_program::check(
        root,
        &folder,
        "original-retained",
        &crate::provider_traits::lower(&original)?,
        &crate::resolve::fixture("pa31/tests/spec/source-assign-ten/constant-observer.c")?,
        "23\n",
    )?;
    Ok(())
}
