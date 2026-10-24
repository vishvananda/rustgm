//! Imported diagnostics retain metadata labels without inventing source units.
use super::{
    catalog::Limits,
    tests::{definitions, input},
    *,
};
use crate::{
    resolve::{
        package::{DiagnosticFile, Error, ErrorData},
        unit::FileId,
    },
    support::crates::owner_catalog::BodyRef,
};

#[test]
fn provider_errors_bypass_source_expansion_and_keep_stable_metadata_coordinates()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let mut catalog = Catalog::new(Limits::default())?;
    let mut request = input("diagnostics", 0, 2)?;
    request.labels[1] = None;
    let installed = catalog.install(&definitions(2), request)?;
    let body = installed.body(BodyRef(0))?;
    let proof = catalog.proof_site(body, catalog.site(body, 0)?)?;
    let origin = catalog.origin(body)?;
    let origins = crate::body::BodyOrigins::from_catalog(&catalog);
    let mut rendered = Vec::new();
    origins.body(origin, &mut rendered)?;
    assert_eq!(
        String::from_utf8(rendered)?,
        "crate:provider[diagnostics]:body:0:site:0"
    );
    assert!(
        crate::body::BodyOrigins::source_only()
            .body(origin, &mut Vec::new())
            .is_err()
    );
    assert_eq!(
        size_of::<crate::body::BodyOrigins<'_>>(),
        size_of::<usize>()
    );
    let error = Error::imported(catalog.diagnostic(proof)?, "checked failure");
    assert_eq!(error.file.source(), None);
    assert_eq!((error.span.start, error.span.end), (2, 8));
    assert_eq!(error.to_string(), "file provider.rs: 2: checked failure");
    assert_eq!(
        report.diagnostic(error).to_string(),
        "file provider.rs: 2: checked failure"
    );
    let second = installed.body(BodyRef(1))?;
    let second_site = catalog.proof_site(second, catalog.site(second, 0)?)?;
    let fallback = Error::imported(catalog.diagnostic(second_site)?, "checked failure");
    assert_eq!(
        fallback.to_string(),
        "file crate provider[diagnostics] body 1 site 0: 0: checked failure"
    );
    let mut foreign = Catalog::new(Limits::default())?;
    foreign.install(&definitions(2), input("diagnostics", 0, 2)?)?;
    assert!(
        crate::body::BodyOrigins::from_catalog(&foreign)
            .body(origin, &mut Vec::new())
            .is_err()
    );
    assert!(foreign.diagnostic(proof).is_err());
    assert!(foreign.diagnostic(second_site).is_err());
    drop(catalog);
    assert!(foreign.diagnostic(second_site).is_err());
    let source = Error::at(FileId(0), 2, "source failure");
    assert_eq!(source.file.source(), Some(FileId(0)));
    assert_eq!(
        report.diagnostic(source).to_string(),
        "file 0: 2: source failure"
    );
    let unavailable = report.diagnostic(Error::unlocated("missing imported owner"));
    assert_eq!(unavailable.file.source(), None);
    assert_eq!(unavailable.to_string(), "input: missing imported owner");
    Ok(())
}

#[test]
fn diagnostic_payload_growth_stays_outside_values_bodies_and_result_abi() {
    struct SourceError {
        file: FileId,
        span: crate::syntax::token_tree::Span,
        message: String,
        expansions: Vec<crate::resolve::package::ExpansionTrace>,
        introduction: crate::expand::marks::ExpansionId,
    }
    assert_eq!(size_of::<Error>(), size_of::<usize>());
    assert_eq!(
        size_of::<Result<(), Error>>(),
        size_of::<Result<(), Box<SourceError>>>()
    );
    assert_eq!(
        size_of::<Result<crate::sema::values::Constant, Error>>(),
        size_of::<Result<crate::sema::values::Constant, Box<SourceError>>>()
    );
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    println!(
        "Error={} DiagnosticFile={} ErrorData={}/{}",
        size_of::<Error>(),
        size_of::<DiagnosticFile>(),
        size_of::<SourceError>(),
        size_of::<ErrorData>()
    );
}
