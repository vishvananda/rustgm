//! Provided library methods enter the ordinary original-template proof inventory.
use super::{
    ir::{
        Action, Body, Local, LocalId, LocalKind, Owner, Place, Rvalue, Statement, Value, ValueId,
        ValueList,
    },
    regions::Contract,
    templates::{Template, Templates},
};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{associated::LibraryDefault, model::Declarations},
    types::{Kind, TypeId},
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../tests/invariants/provided_methods.rs"]
mod tests;

pub(crate) fn append(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut Templates,
    contracts: &mut Vec<Contract>,
) -> Result<(), Error> {
    let mut defaults = facts.associated.library_defaults().collect::<Vec<_>>();
    defaults.sort_unstable_by_key(|(definition, _)| definition.0);
    for (definition, LibraryDefault::PartialEqNe { eq }) in defaults {
        let function = facts
            .associated
            .language_functions()
            .find(|function| function.definition == definition)
            .copied()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing provided library method declaration",
                )
            })?;
        let source = function.node;
        let error = |message| Declarations::error(report, source, message);
        if facts.associated.comparison_method(eq) != Some(crate::support::comparisons::Method::Eq)
            || facts.associated.comparison_method(definition)
                != Some(crate::support::comparisons::Method::Ne)
            || facts.associated.trait_owner(eq) != facts.associated.trait_owner(definition)
        {
            return Err(error(
                "provided equality body has a different canonical owner",
            ));
        }
        let environment = Arc::new(
            facts
                .generics
                .parameter_environment(&facts.types, definition)
                .map_err(error)?,
        );
        let body = build(facts, definition, eq, function.signature, source).map_err(error)?;
        let query = crate::types::destruction::NeedsDrop::new(
            facts.destructors.implementations().iter().map(|i| i.item),
        );
        let destruction =
            super::drops::requirements(facts, &body, &mut query.scoped(Arc::clone(&environment)))
                .map_err(error)?;
        contracts.push(
            super::regions::callable(facts, definition, definition, source, function.signature)
                .map_err(error)?,
        );
        templates.bodies.push(Template {
            body,
            destruction,
            environment,
        });
    }
    Ok(())
}
struct EqualityCall {
    parameters: [TypeId; 2],
    result: TypeId,
    callee: TypeId,
    signature: TypeId,
}
fn call(
    facts: &mut Declarations,
    eq: DefId,
    signature: TypeId,
) -> Result<EqualityCall, &'static str> {
    if !matches!(facts.definitions.get(eq.0 as usize).copied().flatten().and_then(|ty|facts.types.kind(ty)),Some(Kind::FunctionItem{item,signature:declared,..}) if item==crate::types::ItemId(eq.0) && declared==signature)
    {
        return Err("provided equality callee differs from its declared signature");
    }
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
        bound_regions: 2,
    }) = facts.types.kind(signature)
    else {
        return Err("provided equality method has incorrect signature");
    };
    let [left, right] = facts
        .types
        .list(parameters)
        .ok_or("missing equality parameters")?
    else {
        return Err("provided equality method has incorrect arity");
    };
    let [left, right] = [*left, *right];
    let mut arguments = Vec::new();
    for (slot, ty) in [left, right].into_iter().enumerate() {
        let Some(Kind::Reference {
            referent,
            mutable: false,
            region,
        }) = facts.types.kind(ty)
        else {
            return Err("provided equality operand requires a shared borrow");
        };
        if region
            != crate::types::RegionId::bound(
                0,
                u16::try_from(slot).map_err(|_| "invalid equality slot")?,
            )?
        {
            return Err("provided equality operands share a borrow slot");
        }
        arguments.push(crate::types::arguments::Argument::Type(referent));
    }
    if result != TypeId::primitive(crate::types::Primitive::Bool) {
        return Err("provided equality result is not bool");
    }
    let arguments = facts.types.intern_arguments(&arguments)?;
    let callable = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(eq.0),
        signature,
        arguments,
    })?;
    let opened = facts.types.intern(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
        bound_regions: 0,
    })?;
    Ok(EqualityCall {
        parameters: [left, right],
        result,
        callee: callable,
        signature: opened,
    })
}
fn build(
    facts: &mut Declarations,
    definition: DefId,
    eq: DefId,
    signature: TypeId,
    source: crate::resolve::unit::NodeKey,
) -> Result<Body, &'static str> {
    let EqualityCall {
        parameters: [left, right],
        result,
        callee: callable,
        signature: opened,
    } = call(facts, eq, signature)?;
    let mut body = Body::new(
        Owner::Function {
            definition,
            signature,
        },
        source,
        result,
    );
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        source.node,
    )));
    for (index, ty) in [left, right].into_iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| "too many equality parameters")?;
        body.locals.push(Local {
            scope: super::scopes::Id(0),
            storage_parent: index.checked_sub(1).map(LocalId),
            ty,
            kind: LocalKind::Parameter(index),
            definition: None,
            mutable: false,
            source: source.node,
        });
        statement(&mut body, Action::StorageLive(LocalId(index)));
        let value = value(&mut body, ty, Rvalue::Read(Place::Local(LocalId(index))))?;
        body.arguments.push(value);
    }
    let callee = value(&mut body, callable, Rvalue::Function(eq))?;
    let equal = value(
        &mut body,
        result,
        Rvalue::Call {
            signature: opened,
            callee,
            arguments: ValueList { start: 0, end: 2 },
        },
    )?;
    let unequal = value(
        &mut body,
        result,
        Rvalue::Unary {
            operator: crate::syntax::ast::Unary::Not,
            operand: equal,
            literal: false,
        },
    )?;
    super::build::finish_factory(&mut body, unequal)?;
    Ok(body)
}
fn statement(body: &mut Body, action: Action) {
    body.statements.push(Statement {
        source: body.source.node,
        action,
    });
}
fn value(body: &mut Body, ty: TypeId, kind: Rvalue) -> Result<ValueId, &'static str> {
    let id = ValueId(u32::try_from(body.values.len()).map_err(|_| "too many equality values")?);
    body.values.push(Value { ty, kind });
    statement(body, Action::Evaluate(id));
    Ok(id)
}
