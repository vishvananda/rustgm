//! Exact canonical slice originals share ordinary body construction and proof.
use super::{
    ir::{
        Action, Body, Local, LocalId, LocalKind, Owner, Place, Projection, ProjectionId,
        ProjectionKind, Rvalue, Statement, Value, ValueId,
    },
    regions::Contract,
};
use crate::{
    sema::model::{Declarations, Function},
    types::{Kind, Primitive, TypeId},
};
#[cfg(test)]
#[path = "../../tests/invariants/length_contracts.rs"]
mod tests;
pub(crate) fn contract(
    facts: &Declarations,
    function: &Function,
) -> Result<Contract, &'static str> {
    let length = facts
        .methods
        .demanded_length()
        .filter(|l| l.definition == function.definition)
        .ok_or("library original is not a demanded registered method")?;
    length.validate(facts, function)?;
    super::regions::callable(
        facts,
        length.definition,
        length.definition,
        length.origin,
        length.signature,
    )
}
pub(crate) fn append_contract(
    facts: &Declarations,
    bodies: &mut Vec<Contract>,
) -> Result<(), &'static str> {
    if let Some(length) = facts.methods.demanded_length() {
        let function = facts
            .function_indices
            .get(&length.definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("missing demanded slice method function")?;
        bodies.push(contract(facts, function)?);
    }
    Ok(())
}
pub(crate) fn build(
    facts: &Declarations,
    function: &Function,
) -> Result<Option<Body>, &'static str> {
    let Some(length) = facts
        .methods
        .demanded_length()
        .filter(|l| l.definition == function.definition)
    else {
        return Ok(None);
    };
    length.validate(facts, function)?;
    let Some(Kind::Reference {
        referent,
        region,
        mutable: false,
    }) = facts.types.kind(length.receiver)
    else {
        return Err("slice length receiver lost reference identity");
    };
    let result = TypeId::primitive(Primitive::Usize);
    let mut body = Body::new(
        Owner::Function {
            definition: length.definition,
            signature: length.signature,
        },
        length.origin,
        result,
    );
    body.const_context = true;
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        length.origin.node,
    )));
    body.locals.push(Local {
        scope: super::scopes::Id(0),
        storage_parent: None,
        ty: length.receiver,
        kind: LocalKind::Parameter(0),
        definition: None,
        mutable: false,
        source: length.origin.node,
    });
    body.statements.push(Statement {
        source: length.origin.node,
        action: Action::StorageLive(LocalId(0)),
    });
    body.projections.push(Projection {
        base: Place::Local(LocalId(0)),
        kind: ProjectionKind::Dereference {
            region,
            mutable: false,
        },
        ty: referent,
    });
    body.statements.push(Statement {
        source: length.origin.node,
        action: Action::Project(ProjectionId(0)),
    });
    body.values.push(Value {
        ty: result,
        kind: Rvalue::Length(Place::Projection(ProjectionId(0))),
    });
    body.statements.push(Statement {
        source: length.origin.node,
        action: Action::Evaluate(ValueId(0)),
    });
    super::build::finish_factory(&mut body, ValueId(0))?;
    Ok(Some(body))
}
