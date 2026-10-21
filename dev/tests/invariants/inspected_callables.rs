//! The view reports completed closure identities without granting const-call permission.
use super::callable_row;
#[test]
fn closure_callable_retains_its_identity_and_signature() -> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn owner(){let _closure=|x:u8|x+1;}", true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let closure = analysis.facts.closures.all().first().ok_or("closure")?;
    let ty = analysis
        .facts
        .node(closure.source)
        .and_then(|node| node.ty)
        .ok_or("closure type")?;
    let Some(crate::types::Kind::Closure {
        identity,
        signature,
        ..
    }) = analysis.facts.types.kind(ty)
    else {
        return Err("typed closure".into());
    };
    let instance = crate::types::instance::Instance::closure(&analysis.facts.types, ty)?;
    assert_eq!(
        callable_row(&analysis.facts, instance)?,
        format!(
            "function closure={} instance={} signature={} const=false",
            identity.0, ty.0, signature.0
        )
    );
    Ok(())
}
