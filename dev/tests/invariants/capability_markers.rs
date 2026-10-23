//! Independently declared capability opponents retain real source role ownership.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture(name: &str) -> Result<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-header-transport/provider-capabilities")
            .join(format!("{name}.rs")),
    )?)
}
fn resolve(source: &str) -> Result<Report> {
    let provider = CrateIdentity::new("minicore", "provider_capabilities")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(provider.clone()),
        mode: Mode::Source { provider },
        ..Default::default()
    };
    Ok(resolve::analyze_expanded(
        resolve::expansion::analyze_with_context(
            InputFile {
                identity: "provider.rs".into(),
                path: "provider.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
            &context,
        )?,
    )?)
}
fn check(source: &str) -> Result {
    let report = resolve(source)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    crate::lower::instances::inspect_prepared(&report, &mut facts, analysis.prepared, |_, _| ())?;
    Ok(())
}

#[test]
fn scoped_source_capabilities_match_the_independent_valid_invalid_and_unsupported_matrix() -> Result
{
    for (name, accepted) in [
        ("scoped-sizing", true),
        ("fundamental-request-only", true),
        ("user-fundamental", false),
        ("lookalike-fundamental", false),
        ("user-coinductive", false),
        ("user-specialization", false),
        ("missing-fundamental-gate", false),
        ("malformed-coinductive", false),
        ("sizing-explicit-impl", false),
        ("const-destruct-request-only", true),
        ("dyn-destruct", false),
        ("dyn-destruct-supertrait", false),
        ("dyn-lookalike", true),
        ("diagnostic-malformed", false),
        ("diagnostic-duplicate", false),
        ("trivial-field-read-no-copy", false),
        ("trivial-field-read-header", true),
    ] {
        let result = check(&fixture(name)?);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        if let Err(error) = result {
            let message = error.to_string();
            if matches!(
                name,
                "user-fundamental"
                    | "lookalike-fundamental"
                    | "user-coinductive"
                    | "user-specialization"
            ) {
                assert!(
                    message.contains("actual language role")
                        || message.contains("actual-role profile"),
                    "{name}: {message}"
                );
            }
            if matches!(name, "dyn-destruct" | "dyn-destruct-supertrait") {
                assert!(
                    message.contains("actual source trait requires dyn incompatibility"),
                    "{name}: {message}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn source_role_attributes_are_cold_and_cannot_validate_foreign_headers_or_pay_goals() -> Result {
    let report = resolve(&fixture("scoped-sizing")?)?;
    let reservations = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?;
    for role in [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized] {
        let flags = reservations.role_attributes(role);
        assert!(flags.requires_sizing_profile());
        for attribute in [
            HeaderAttribute::Fundamental,
            HeaderAttribute::Specialization,
            HeaderAttribute::Coinductive,
        ] {
            assert!(flags.contains(attribute));
        }
        let declaration = reservations.language(role).ok_or("role")?;
        assert!(reservations.denies_implementation(declaration.definition));
    }
    let facts = crate::sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let sized = catalog.language(LangItem::Sized).ok_or("sized")?.definition;
    assert!(catalog.forces_dyn_incompatible(sized)?);
    assert!(!catalog.forces_dyn_incompatible(crate::resolve::model::DefId(u32::MAX))?);
    assert_eq!(size_of::<HeaderAttributes>(), 1);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(
        size_of::<crate::types::parameters::predicates::Predicate>(),
        12
    );
    println!(
        "Reservations={} SourceCatalog={} RoleMask={} RoleSlots={} Owner={} OwnerKind={}",
        size_of::<Reservations>(),
        size_of_val(catalog),
        size_of::<HeaderAttributes>(),
        LangItem::ALL.len(),
        size_of::<crate::sema::associated::Owner>(),
        size_of::<crate::sema::associated::OwnerKind>()
    );
    Ok(())
}

#[test]
fn requested_const_destruct_does_not_gate_or_erase_an_existing_qualified_premise() -> Result {
    check(&fixture("conditional-destruct-bound-only-freeze")?)?;
    let report = crate::test_source::resolve("fn ordinary(){}", true)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    assert!(report.package.source_core.is_none() && report.resolution.source_bindings.is_none());
    assert!(facts.source_bindings.is_none());
    Ok(())
}
