//! A diagnostic annotation cannot erase destruction, moves or lifetimes.
use crate::{pipeline, test_source};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const BASE: &str = "pa31/tests/spec/source-insignificant-dtor/";

#[test]
fn frozen_source_targets_forms_macro_authority_and_real_drop_bodies() -> TestResult {
    for (name, accepted) in [
        ("word", true),
        ("enum", true),
        ("foreign", true),
        ("generic", true),
        ("alias", true),
        ("duplicate", false),
        ("missing-feature", false),
        ("list", false),
        ("value", false),
        ("unsafe-wrapper", false),
        ("wrong-union", false),
        ("wrong-trait", false),
        ("wrong-function", false),
        ("wrong-alias", false),
        ("wrong-impl", false),
        ("wrong-module", false),
        ("wrong-field", false),
        ("wrong-variant", false),
        ("wrong-const", false),
        ("wrong-static", false),
        ("wrong-closure", false),
        ("wrong-foreign-function", false),
        ("macro-authorized", true),
        ("macro-captured-caller", false),
        ("macro-unprivileged", false),
        ("macro-global-request", true),
        ("cfg-removed", true),
        ("not-copy", false),
        ("still-borrowed", false),
        ("with-target-has", true),
        ("still-drops", true),
    ] {
        let source = test_source::fixture(&format!("{BASE}{name}.rs"))?;
        let outcome = test_source::resolve(&source, true)
            .map_err(Into::into)
            .and_then(|report| {
                let pipeline::Analysis {
                    mut facts,
                    prepared,
                } = pipeline::analyze_source(&report)?;
                let program = crate::lower::instances::build_roots_prepared(
                    &report,
                    &mut facts,
                    crate::lower::Roots {
                        entry: None,
                        exports: &[],
                    },
                    prepared,
                )?;
                if name == "still-drops" {
                    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../obj/parallel/insignificant-dtor-v1/candidate");
                    std::fs::create_dir_all(&directory)?;
                    std::fs::write(directory.join("source.rs"), &source)?;
                    crate::lowir::writer::write(
                        &program,
                        &mut std::fs::File::create(directory.join("provider.lowir"))?,
                    )?;
                }
                Ok::<_, Box<dyn std::error::Error>>(())
            });
        if let Err(error) = &outcome {
            println!("insignificant-dtor {name}: {error}");
        }
        assert_eq!(outcome.is_ok(), accepted, "{name}: {:?}", outcome.err());
    }
    Ok(())
}

#[test]
fn duplicate_state_does_not_escape_the_configured_owner_or_require_ordinary_rows() -> TestResult {
    let marked = test_source::fixture(&format!("{BASE}word.rs"))?;
    let source =
        format!("{marked}\n#[rustc_insignificant_dtor]enum Other{{A}}\nstruct Ordinary;\n");
    let report = test_source::resolve(&source, true)?;
    let facts = pipeline::analyze_source(&report)?.facts;
    assert_eq!(
        crate::sema::header_attributes::source_contract_costs(&facts).1,
        0,
        "diagnostic-only hints do not retain source permission rows"
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
