//! Ordinary inherent aliases retain declaration and namespace proof boundaries.
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
const ROOT: &str = "pa20/tests/spec/inherent-associated";
fn analyze(source: &str) -> std::result::Result<(), String> {
    let report = crate::resolve::resolve(source, true)?;
    let facts = rustgm::sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    rustgm::sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    Ok(())
}
fn input(name: &str) -> std::result::Result<String, Box<dyn std::error::Error>> {
    Ok(crate::resolve::fixture(&format!("{ROOT}/{name}.rs"))?)
}
#[test]
fn closed_alias_declarations_and_real_type_uses() -> Result {
    let mut errors = Vec::new();
    for name in [
        "unused",
        "lookup",
        "explicit",
        "self-use",
        "public",
        "specialized",
        "disjoint",
        "unsized",
        "disjoint-kinds",
        "trait-control",
    ] {
        if let Err(error) = analyze(&input(name)?) {
            errors.push(format!("{name}: {error}"));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}
#[test]
fn ordinary_alias_errors_remain_errors() -> Result {
    for (name, cause) in [
        ("missing-feature", "actual feature request"),
        ("missing-result", "requires a definition"),
        ("written-bound", "cannot declare bounds"),
        ("invalid-unused", "unresolved type name"),
        ("wrong-owner", "actual nominal owner"),
        ("private", "no applicable inherent associated type"),
        ("duplicate", "duplicate inherent"),
        ("wrong-result-closed", "incompatible types"),
        ("namespaces", "duplicate inherent"),
        ("overlap", "duplicate inherent"),
    ] {
        let error = analyze(&input(name)?)
            .err()
            .ok_or_else(|| format!("{name} accepted"))?;
        eprintln!("{name}: {error}");
        assert!(error.contains(cause), "{name}: {error}");
        assert!(
            !error.contains("requires its selected owner proof"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn generic_owner_and_member_proof_remain_named_requirements() -> Result {
    for name in [
        "generic-owner",
        "generic-member",
        "lifetime-member",
        "lifetime-owner",
        "where-owner",
        "member-where",
    ] {
        let error = analyze(&input(name)?)
            .err()
            .ok_or("unproved generic alias accepted")?;
        assert!(
            error.contains("requires its selected owner proof"),
            "{name}: {error}"
        );
    }
    Ok(())
}

fn native_program()
-> std::result::Result<rustgm::lowir::program::Program, Box<dyn std::error::Error>> {
    let report = crate::resolve::resolve(&input("native")?, true)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_inherent_alias_body_reaches_ordinary_lowering() -> Result {
    let program = native_program()?;
    assert!(program.symbols.iter().any(|symbol| symbol.body.is_some()));
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn actual_inherent_alias_identity_executes() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/source-transport/inherent-types/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "inherent-types",
        &native_program()?,
        &crate::resolve::fixture(&format!("{ROOT}/observer.c"))?,
        "alias 23\n",
    )?;
    Ok(())
}
