//! Header views retain real declaration facts and expose missing bodies honestly.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_headers_borrow_actual_signature_universals_and_environment() -> Result {
    let report = crate::test_source::resolve(
        "fn id<'a,T:Copy>(x:&'a T)->&'a T{x} unsafe extern \"C\"{fn absent(x:u8)->u8;} fn main(){}",
        true,
    )?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let facts = &analysis.facts;
    let function = facts
        .functions
        .iter()
        .find(|f| facts.generics.has_parameters(f.definition))
        .ok_or("generic source function")?;
    let header = facts
        .function_header(function.definition)?
        .ok_or("source header")?;
    assert_eq!(header.declaration.source(), Some(function.node));
    assert_eq!(header.body, function.body.map(BodyOrigin::Source));
    assert_eq!(header.signature, function.signature);
    assert_eq!(
        header.universals,
        facts.lifetimes.functions[&function.definition].total
    );
    assert!(header.universals > 0);
    assert!(Arc::ptr_eq(
        header.environment.ok_or("header environment")?,
        facts
            .generics
            .body_environment(function.definition)
            .ok_or("source environment")?
    ));
    let absent = facts
        .functions
        .iter()
        .find(|f| f.body.is_none())
        .ok_or("bodyless declaration")?;
    let header = facts
        .function_header(absent.definition)?
        .ok_or("bodyless header")?;
    assert!(header.body.is_none());
    assert_eq!(header.declaration.source(), Some(absent.node));
    assert!(facts.function_header(DefId(u32::MAX))?.is_none());
    Ok(())
}

#[test]
fn header_lookup_rejects_stale_and_foreign_owner_indices() -> Result {
    let report = crate::test_source::resolve("fn a(){} fn b(){} fn main(){}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let first = facts.functions[0].definition;
    let second = facts.functions[1].definition;
    facts
        .function_indices
        .insert(first, facts.function_indices[&second]);
    assert!(facts.function_header(first).is_err());
    facts.function_indices.insert(first, usize::MAX);
    assert!(facts.function_header(first).is_err());
    Ok(())
}

#[test]
fn retained_scope_sites_are_body_owned_without_widening_records() -> Result {
    let report = crate::test_source::resolve("fn f(x:u8)->u8{if x==0{1}else{x}}fn main(){}", true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut prepared = crate::body::templates::Prepared::default();
    let function = analysis.facts.functions.iter().find(|f| {
        matches!(analysis.facts.types.kind(f.signature), Some(crate::types::Kind::Function {result,..})
            if result != crate::types::TypeId::UNIT)
    }).ok_or("source function")?;
    let template = prepared.function(&report, &analysis.facts, function)?;
    let storage = template.body.storage.as_ref().ok_or("storage")?;
    assert!(!storage.scopes.is_empty());
    for scope in &storage.scopes {
        let location = template.body.source.proof_site(scope.source)?;
        assert_eq!(
            location.source().ok_or("source scope")?.file,
            function.node.file
        );
    }
    assert!(storage.scope_work >= storage.scopes.len());
    assert_eq!(size_of::<crate::sema::scopes::Scope>(), 16);
    assert_eq!(size_of::<crate::body::storage::Scope>(), 16);
    Ok(())
}
