//! Bounds make a diagnostic carrier, never an original or execution permission.
use super::{
    catalog::{Expected, Limits},
    tests::{definitions, input},
    *,
};
use crate::{
    resolve::model::DefId,
    support::crates::owner_catalog::{BodyKind, BodyRef},
};

#[test]
fn checked_origins_compose_only_the_exact_catalog_owned_sites() -> Result<(), &'static str> {
    let mut catalog = Catalog::new(Limits::default())?;
    let body = catalog
        .install(&definitions(1), input("bounds", 0, 1)?)?
        .body(BodyRef(0))?;
    let expected = Expected {
        definition: DefId(0),
        kind: BodyKind::Function,
        parent: None,
        unit: body,
    };
    let origin = catalog.validate(body, &expected)?;
    assert_eq!(origin, catalog.origin(body)?);
    catalog.validate_origin(origin, &expected)?;
    let site = catalog.site(body, 0)?;
    let work = catalog.work();
    let proof = origin.proof_site(site)?;
    assert_eq!(catalog.work(), work); // No owner/catalog scan during composition.
    assert_eq!(proof, catalog.proof_site(body, site)?);
    assert!(origin.proof_site(BodySite::checked(1)?).is_err());
    let BodyOrigin::Imported(checked) = origin else {
        return Err("missing imported carrier");
    };
    assert_eq!(checked.id(), body);
    let altered = BodyOrigin::Imported(ImportedBodyOrigin {
        sites: NonZeroU32::MIN.saturating_add(1),
        ..checked
    });
    assert!(catalog.validate_origin(altered, &expected).is_err());
    let mut foreign = Catalog::new(Limits::default())?;
    foreign.install(&definitions(1), input("bounds", 0, 1)?)?;
    assert!(foreign.validate_origin(origin, &expected).is_err());
    assert!(foreign.location(proof).is_err());
    drop(catalog);
    let mut replacement = Catalog::new(Limits::default())?;
    replacement.install(&definitions(1), input("bounds", 0, 1)?)?;
    // A retained location can still compose diagnostics; it cannot authorize a
    // current original or resolve against a new catalog with equal numeric rows.
    assert_eq!(origin.proof_site(site)?, proof);
    assert!(replacement.validate_origin(origin, &expected).is_err());
    assert!(replacement.location(proof).is_err());
    Ok(())
}

#[test]
fn empty_executable_functions_have_real_sites_without_an_extra_count_encoding()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let program = crate::body::build(&report, &analysis.facts)?;
    let body = program
        .bodies
        .first()
        .ok_or("empty source function missing")?;
    assert_eq!(body.blocks.len(), 1);
    let source = body.source;
    assert_eq!(
        source.proof_site(body.blocks[0].source)?.source(),
        Some(body.syntax_site(body.blocks[0].source)?)
    );
    let mut catalog = Catalog::new(Limits::default())?;
    let id = catalog
        .install(&definitions(1), input("empty", 0, 1)?)?
        .body(BodyRef(0))?;
    let BodyOrigin::Imported(origin) = catalog.origin(id)? else {
        return Err("missing body origin".into());
    };
    assert_eq!(origin.sites.get(), 1);
    assert_eq!(catalog.site(id, 0)?.index(), 0);
    assert!(catalog.site(id, 1).is_err());
    Ok(())
}
