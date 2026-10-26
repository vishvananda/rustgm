//! Direct aliases feed real originals, concrete instance proof and `LowIR` lowering.
use super::*;

fn lower(report: &Report, root: &str) -> TestResult<crate::lowir::program::Program> {
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(report)?;
    let root = definition(report, root, DefKind::Function)?;
    Ok(crate::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[root],
        },
        prepared,
    )?)
}

#[test]
fn source_thin_unit_and_ordinary_copy_reach_complete_body_and_instance_checks() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/trait-aliases/source-thin/thin-unit.rs")?;
    let report = crate::test_source::source_provider(&source, "thin_provider")?;
    let program = lower(&report, "concrete")?;
    assert!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count()
            >= 2
    );
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-aliases/declaration-formation/local-copy-consumer.rs",
    )?;
    let program = lower(&super::report(&source)?, "value")?;
    assert!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count()
            >= 2
    );
    Ok(())
}

#[test]
fn source_thin_wide_pointer_cannot_supply_unit_metadata_evidence() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/trait-aliases/source-thin/thin-wide.rs")?;
    let report = crate::test_source::source_provider(&source, "thin_provider")?;
    let error = lower(&report, "concrete")
        .err()
        .ok_or("wide pointer satisfied Thin")?;
    let message = error.to_string();
    assert!(message.ends_with("incompatible types"), "{message}");
    eprintln!("actual wide Thin proof rejection: {message}");
    Ok(())
}

#[test]
fn native_thin_provider_lowers_while_public_alias_metadata_remains_closed() -> TestResult {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/trait-aliases/source-thin/native-provider.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "thin_provider")?;
    let program = lower(&report, "source_thin_result")?;
    assert!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count()
            >= 2
    );
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../obj/parallel/direct-thin-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), &source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;

    let public = super::report("#![feature(trait_alias)]pub trait Alias=Copy;")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&public)?;
    let error = crate::lower::instances::build_library_prepared(
        &public,
        &mut facts,
        crate::support::crates::CrateIdentity::new("alias_provider", "direct")?,
        prepared,
    )
    .err()
    .ok_or("public alias acquired an ordinary trait transport identity")?;
    assert!(
        error
            .message
            .contains("exposed declaration is outside the scalar metadata profile"),
        "{error}"
    );
    Ok(())
}
