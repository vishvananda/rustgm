//! Public callable and opened body roots retain independent region identities.
use super::*;
use crate::types::{ItemId, Primitive, RegionId, arguments::Argument};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn public_signature_prepares_distinct_binder_storage_without_rewriting() -> Result {
    let report = crate::test_source::resolve(
        "struct I;impl Iterator for I{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){for _ in I{}}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let option = ItemId(
        facts
            .iteration
            .as_deref()
            .ok_or("iteration")?
            .canonical
            .option
            .0,
    );
    let element = facts.types.intern(Kind::Primitive(Primitive::U8))?;
    let mut roots = Vec::new();
    for depth in [0, 1] {
        let reference = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(depth, 7)?,
            referent: element,
        })?;
        let arguments = facts.types.intern_arguments(&[Argument::Type(reference)])?;
        roots.push(facts.types.intern(Kind::Adt(option, arguments))?);
    }
    assert_ne!(roots[0], roots[1]);
    facts.aggregates.prepare(&mut facts.types, roots[0])?;
    assert!(facts.aggregates.shape(&facts.types, roots[1]).is_err());
    let parameters = facts.types.intern_list(&[roots[1]])?;
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result: roots[1],
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
        bound_regions: 8,
    })?;
    let owner = Owner::Function {
        definition: DefId(0),
        signature,
    };
    prepare_signature(&mut facts, owner)?;
    facts.aggregates.shape(&facts.types, roots[0])?;
    facts.aggregates.shape(&facts.types, roots[1])?;
    let steps = facts.aggregates.instantiation_steps();
    prepare_signature(&mut facts, owner)?;
    assert_eq!(facts.aggregates.instantiation_steps(), steps);
    assert!(matches!(owner, Owner::Function { signature: actual, .. } if actual == signature));
    assert!(
        matches!(facts.types.kind(signature), Some(Kind::Function { parameters: actual, result, bound_regions: 8, .. }) if actual == parameters && result == roots[1])
    );
    assert!(
        prepare_signature(
            &mut facts,
            Owner::Function {
                definition: DefId(0),
                signature: TypeId::UNIT,
            }
        )
        .is_err()
    );
    Ok(())
}
