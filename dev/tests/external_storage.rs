//! Foreign storage retains canonical identity, safety and real declaration state.
use crate::foreign_frontend as frontend;
#[cfg(feature = "object-interop-tests")]
use crate::host_objects as objects;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
const CASES: &[(&str, bool, i32)] = &[
    ("default-unsafe", true, 0),
    ("default-safe-use", false, 1),
    ("explicit-safe", true, 0),
    ("explicit-unsafe", true, 0),
    ("safe-mutable", true, 0),
    ("mutable", true, 0),
    ("mutable-raw-address", true, 0),
    ("immutable-raw-address", true, 0),
    ("mutable-reference", false, 1),
    ("immutable-reference", true, 0),
    ("export-static", true, 0),
    ("self-alias", true, 0),
    ("extern-rust", true, 0),
    ("foreign-initializer", false, 1),
    ("const-read", false, 1),
    ("const-address", true, 0),
    ("ordinary-safe", false, 1),
    ("ordinary-unsafe", false, 1),
    ("generated-collision", false, 0),
    ("exact-kind-collision", false, 1),
    ("safe-mutable-use", false, 1),
    ("safe-immutable-reference", true, 0),
    ("unsafe-raw-dereference", false, 1),
    ("safe-mutable-reference", false, 1),
    ("generated-import-collision", false, 0),
    ("imports", true, 0),
    ("exports", true, 0),
    ("alias", true, 0),
    ("callback", true, 0),
    ("pointer-constant", true, 0),
    ("interior-storage", true, 0),
];
fn root() -> Result<&'static Path, Box<dyn std::error::Error>> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
fn fixture(name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(root()?.join(format!("pa30/tests/spec/statics/{name}.rs")))
}
#[test]
fn foreign_static_outcomes_preserve_independent_safety_rules()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = root()?.join(format!(
        "obj/parallel/foreign-statics/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, accepted, status) in CASES {
        let input = fixture(name)?;
        let independent = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=static_case",
                "--emit=obj",
            ])
            .arg(&input)
            .arg("-o")
            .arg(folder.join(format!("{name}.o")))
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.rustc.stderr")),
            &independent.stderr,
        )?;
        assert_eq!(
            independent.status.code(),
            Some(status),
            "rustc {name}: {}",
            String::from_utf8_lossy(&independent.stderr)
        );
        let result = frontend::checked(&std::fs::read_to_string(input)?);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.as_ref().err()
        );
        if status == 0 && !accepted {
            assert!(
                result
                    .err()
                    .is_some_and(|e| e.contains("LowIR symbol kind")),
                "{name}"
            );
        }
    }
    Ok(())
}
#[test]
fn imported_storage_has_no_local_initializer_or_definition()
-> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(fixture("imports")?)?;
    let (_, analysis) = frontend::analyze(&source)?;
    let registry = analysis.facts.foreign.as_deref().ok_or("registry")?;
    for (index, _) in analysis.facts.definitions.iter().enumerate() {
        let id = rustgm::resolve::model::DefId(u32::try_from(index)?);
        if registry.imports_global(id) {
            assert!(
                analysis
                    .facts
                    .initializers
                    .iter()
                    .all(|i| i.definition != id)
            );
        }
    }
    let program = frontend::checked(&source)?;
    let imports: Vec<_> = program
        .globals
        .iter()
        .filter(|g| g.kind == rustgm::lowir::data::GlobalKind::Declaration)
        .collect();
    assert_eq!(imports.len(), 6);
    assert_eq!(
        imports
            .iter()
            .filter(|g| g.name == "provided_readonly")
            .count(),
        1
    );
    assert!(imports.iter().all(|g| g.items.is_none()));
    Ok(())
}

#[test]
fn imported_shared_storage_uses_its_owned_mutation_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let program = frontend::checked(&std::fs::read_to_string(fixture("interior-storage")?)?)?;
    for (name, readonly) in [("foreign_atomic", false), ("foreign_ordinary", true)] {
        let global = program
            .globals
            .iter()
            .find(|global| global.name == name)
            .ok_or("missing imported storage")?;
        assert_eq!(global.kind, rustgm::lowir::data::GlobalKind::Declaration);
        assert_eq!(global.span.bytes, 4);
        assert_eq!(global.span.align, 4);
        assert_eq!(global.readonly, readonly);
    }
    Ok(())
}
#[test]
#[cfg(feature = "object-interop-tests")]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn foreign_globals_and_initializer_callbacks_interoperate_with_c()
-> Result<(), Box<dyn std::error::Error>> {
    objects::check_cases(
        "foreign-statics/imports",
        "pa30/tests/spec/statics/imports.c",
        &[
            ("imports", "pa30/tests/spec/statics/imports.rs"),
            ("alias", "pa30/tests/spec/statics/alias.rs"),
            (
                "pointer-constant",
                "pa30/tests/spec/statics/pointer-constant.rs",
            ),
        ],
    )?;
    objects::check_cases(
        "foreign-statics/exports",
        "pa30/tests/spec/statics/exports.c",
        &[("exports", "pa30/tests/spec/statics/exports.rs")],
    )?;
    objects::check_cases(
        "foreign-statics/callback",
        "pa30/tests/spec/statics/callback.c",
        &[("callback", "pa30/tests/spec/statics/callback.rs")],
    )
}
