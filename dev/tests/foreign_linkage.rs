//! Exact linkage retains language identities, root bodies and spelling boundaries.
use crate::foreign_frontend as frontend;
use crate::host_objects as objects;
use std::{path::Path, process::Command};
const CASES: &[(&str, bool, bool)] = &[
    ("tests/spec/linkage/no-mangle.rs", true, true),
    ("tests/spec/linkage/export-name.rs", true, true),
    ("tests/spec/linkage/alias.rs", true, true),
    ("tests/spec/linkage/collision.rs", false, false),
    ("tests/spec/linkage/missing-unsafe.rs", false, false),
    ("tests/spec/linkage/generic.rs", false, false),
    ("tests/spec/linkage/generic-export.rs", false, false),
    ("tests/spec/linkage/generic-const.rs", false, false),
    ("tests/spec/linkage/empty.rs", false, false),
    ("tests/spec/linkage/nul.rs", false, false),
    ("tests/spec/linkage/unicode.rs", true, false),
    ("tests/spec/linkage/unsupported-byte.rs", true, false),
    ("tests/spec/linkage/duplicate-export.rs", true, true),
    ("tests/spec/linkage/both-export.rs", true, true),
    ("tests/spec/linkage/wrong-owner.rs", true, true),
    ("tests/spec/linkage/foreign-export.rs", false, false),
    ("tests/spec/linkage/helper-first.rs", true, false),
    ("tests/spec/linkage/export-first.rs", true, false),
    ("tests/spec/linkage/exports.rs", true, true),
    ("tests/spec/linkage/import-alias.rs", true, true),
];
fn root() -> Result<&'static Path, Box<dyn std::error::Error>> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "root".into())
}
fn fixture(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(std::fs::read_to_string(root()?.join("pa30").join(path))?)
}
#[test]
fn linkage_outcomes_distinguish_language_and_backend_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = root()?.join(format!(
        "obj/parallel/foreign-linkage/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(path, language, accepted) in CASES {
        let source = fixture(path)?;
        let name = Path::new(path)
            .file_stem()
            .ok_or("stem")?
            .to_str()
            .ok_or("name")?;
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, &source)?;
        let output = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=linkage_case",
                "--emit=obj",
            ])
            .arg(&input)
            .arg("-o")
            .arg(folder.join(format!("{name}.o")))
            .output()?;
        std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(!language)),
            "rustc {name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result = frontend::checked(&source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.as_ref().err()
        );
        if language && !accepted {
            let diagnostic = result.err().ok_or("expected backend boundary")?;
            assert!(
                diagnostic.contains(if matches!(name, "unicode" | "unsupported-byte") {
                    "pinned LowIR vocabulary"
                } else {
                    "multiple function definitions"
                }),
                "{name}: {diagnostic}"
            );
        }
    }
    Ok(())
}
#[test]
fn exact_metadata_matches_selected_attribute_precedence() -> Result<(), Box<dyn std::error::Error>>
{
    for (path, expected) in [
        ("duplicate-export.rs", "first"),
        ("both-export.rs", "selected"),
    ] {
        let source = fixture(&format!("tests/spec/linkage/{path}"))?;
        let (_, analysis) = frontend::analyze(&source)?;
        let registry = analysis
            .facts
            .foreign
            .as_deref()
            .ok_or("missing registry")?;
        let exports = registry.exports();
        assert_eq!(exports.len(), 1);
        let function = registry.function(exports[0]).ok_or("missing export")?;
        let rustgm::support::linkage::Naming::Exact(id) = function.naming else {
            return Err("missing exact identity".into());
        };
        assert_eq!(registry.names().get(id), Some(expected));
    }
    Ok(())
}
#[test]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn exact_exports_and_import_aliases_link_with_c() -> Result<(), Box<dyn std::error::Error>> {
    objects::check_cases(
        "foreign-linkage/imports",
        "pa30/tests/spec/imports/provider.c",
        &[("alias", "pa30/tests/spec/linkage/import-alias.rs")],
    )?;
    objects::check_cases(
        "foreign-linkage/exports",
        "pa30/tests/spec/linkage/provider.c",
        &[("exports", "pa30/tests/spec/linkage/exports.rs")],
    )
}
