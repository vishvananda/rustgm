//! Execution matching keeps body slots separate from nested public binders.
use super::*;
use crate::{
    sema::closures::Execution,
    types::{RegionId, Store, callable::Convention},
};
type Result<T = ()> = std::result::Result<T, &'static str>;
fn reference(types: &mut Store, region: RegionId) -> Result<TypeId> {
    types.intern(Type::Reference {
        region,
        referent: TypeId::primitive(crate::types::Primitive::U8),
        mutable: false,
    })
}
fn function(types: &mut Store, inputs: &[TypeId], bound_regions: u32) -> Result<TypeId> {
    let parameters = types.intern_list(inputs)?;
    types.intern(Type::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
fn formal(signature: TypeId) -> Execution {
    Execution {
        self_ty: TypeId::UNIT,
        signature,
        universals: 1,
        receiver: CallKind::Shared,
    }
}
#[test]
fn repeated_execution_slots_retain_caller_equalities() -> Result {
    let mut types = Store::default();
    let expected = reference(&mut types, RegionId::bound(0, 0)?)?;
    let signature = function(&mut types, &[expected, expected], 1)?;
    let a = reference(&mut types, RegionId(7))?;
    let b = reference(&mut types, RegionId(9))?;
    let selected = function(&mut types, &[a, b], 0)?;
    let pairs = matching::execution(&types, formal(signature), selected)?;
    assert_eq!(pairs.len(), 1);
    assert!(pairs[0] == (RegionId(7), RegionId(9)) || pairs[0] == (RegionId(9), RegionId(7)));
    Ok(())
}
#[test]
fn nested_binders_are_rigid_and_cannot_escape_into_body_slots() -> Result {
    let mut types = Store::default();
    let local = reference(&mut types, RegionId::bound(0, 0)?)?;
    let inherited = reference(&mut types, RegionId::bound(1, 0)?)?;
    let concrete = reference(&mut types, RegionId(7))?;
    let nested = function(&mut types, &[local, inherited], 1)?;
    let valid = function(&mut types, &[local, concrete], 1)?;
    let escaped = function(&mut types, &[local, local], 1)?;
    let source = function(&mut types, &[nested], 1)?;
    let actual = function(&mut types, &[valid], 0)?;
    assert!(matching::execution(&types, formal(source), actual)?.is_empty());
    let actual = function(&mut types, &[escaped], 0)?;
    assert!(matching::execution(&types, formal(source), actual).is_err());
    let other = reference(&mut types, RegionId::bound(0, 1)?)?;
    let changed = function(&mut types, &[other, concrete], 1)?;
    let actual = function(&mut types, &[changed], 0)?;
    assert!(matching::execution(&types, formal(source), actual).is_err());
    Ok(())
}
#[test]
fn execution_matching_rejects_changed_types_and_convention() -> Result {
    let mut types = Store::default();
    let a = reference(&mut types, RegionId::bound(0, 0)?)?;
    let source = function(&mut types, &[a], 1)?;
    let different = function(&mut types, &[TypeId::UNIT], 0)?;
    assert!(matching::execution(&types, formal(source), different).is_err());
    let Type::Function {
        parameters, result, ..
    } = types.kind(different).ok_or("signature")?
    else {
        return Err("function");
    };
    let c = types.intern(Type::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::C,
    })?;
    assert!(matching::execution(&types, formal(source), c).is_err());
    Ok(())
}
#[test]
fn closure_proof_requires_the_exact_execution_signature()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){let x=7u8;let f=||x;let _=f();}", true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let program = crate::body::build_concrete(&report, &facts)?;
    let body = program
        .bodies
        .iter()
        .find(|body| {
            matches!(
                body.owner,
                crate::body::ir::Owner::Closure {
                    execution: Some(_),
                    ..
                }
            )
        })
        .ok_or("captured body")?;
    let mut inputs = crate::body::regions::collect(&facts)?;
    let input = inputs
        .bodies
        .iter_mut()
        .find(|input| input.key == body.key())
        .ok_or("contract")?;
    input.signature = Some(TypeId::UNIT);
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let error = crate::borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        input,
        body,
    )
    .err()
    .ok_or("accepted wrong execution signature")?;
    assert!(error.message.contains("checked execution signature"));
    Ok(())
}
