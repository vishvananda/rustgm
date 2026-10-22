//! Const-function factories construct values without gaining variant promotion permissions.
use crate::native_references as native;
use crate::qualify as independent;
use std::{path::Path, process::Command};
const CASES: &[(&str, bool)] = &[
    ("runtime-u32", false),
    ("const-u32", true),
    ("runtime-option", false),
    ("runtime-unit", false),
    ("named-const-u32", true),
    ("variant-control", true),
    ("user-const-function-control", false),
    ("lookalike-control", true),
    ("result-control", true),
    ("tuple-control", true),
    ("alias-runtime", false),
    ("const-runtime-value", true),
];
fn fixture(name: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .join(format!("pa26/tests/spec/constructor-promotion/{name}.rs")))
}
#[test]
fn constructor_permissions_match_independent_lifetime_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/constructor-promotions/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, accepted) in CASES {
        let path = fixture(name)?;
        let source = std::fs::read_to_string(&path)?;
        independent::qualify(
            &format!("parallel/constructor-promotions/qualification/{name}"),
            &source,
            accepted,
        )?;
        let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&path)
            .output()?;
        std::fs::write(
            folder.join(format!("{name}.frontend.stderr")),
            &result.stderr,
        )?;
        assert_eq!(
            result.status.code(),
            Some(i32::from(!accepted)),
            "{name}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn const_construction_and_genuine_variant_promotion_execute()
-> Result<(), Box<dyn std::error::Error>> {
    let sources: Vec<_> = [
        "const-u32",
        "named-const-u32",
        "lookalike-control",
        "result-control",
        "const-runtime-value",
    ]
    .into_iter()
    .map(|name| Ok((name, std::fs::read_to_string(fixture(name)?)?)))
    .collect::<Result<_, Box<dyn std::error::Error>>>()?;
    let cases: Vec<_> = sources
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect();
    native::check_cases("parallel/constructor-promotions/cases", &cases)
}
