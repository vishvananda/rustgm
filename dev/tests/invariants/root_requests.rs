//! The selected initializer route cannot authorize a different expression root.
use super::*;
use crate::{
    sema::constants::instances::Instance,
    syntax::ast::Kind,
    types::{GenericArgs, Kind as Type, RegionId, arguments::Argument},
};

#[test]
fn nonselected_expression_keeps_strict_projected_type_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve(
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}struct S<'a,T>(&'a u8,T);impl<'a,T:Pick<'a>>S<'a,T>{const C:T::Out={let n=T::V;n};}fn main(){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let initializer = *facts.initializers.last().ok_or("missing initializer")?;
    let tag = facts
        .definitions
        .iter()
        .flatten()
        .copied()
        .find(|&ty| matches!(facts.types.kind(ty), Some(Type::Adt(_, GenericArgs::EMPTY))))
        .ok_or("missing closed nominal")?;
    let arguments = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 0)?),
        Argument::Type(tag),
    ])?;
    let instance = Instance {
        definition: initializer.definition,
        arguments,
    };
    let Kind::Block {
        tail: Some(tail), ..
    } = crate::sema::model::Declarations::kind(&report, initializer.value)
    else {
        return Err("missing actual tail expression".into());
    };
    let child = NodeKey {
        node: tail,
        ..initializer.value
    };
    let before = facts.constant_work.source_context_bytes;
    let mut queries = Queries::new(&report, &mut facts);
    queries.instance_source(instance)?;
    let error = queries
        .execution_target(child, initializer.ty)
        .err()
        .ok_or("unselected projection acquired source authority")?;
    assert!(
        error
            .to_string()
            .contains("constant instance projection normalization"),
        "{error}"
    );
    assert_eq!(
        queries.typing.facts.constant_work.source_context_bytes,
        before
    );
    Ok(())
}
