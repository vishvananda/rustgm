//! Provided library methods enter the ordinary original-template proof inventory.
mod extrema;
mod ordering;
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
    for (definition, default) in defaults {
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
        let error = |message| report.body_error(source, message);
        validate_default(facts, definition, default).map_err(error)?;
        let environment = Arc::new(
            facts
                .generics
                .parameter_environment(&facts.types, definition)
                .map_err(error)?,
        );
        let mut contract =
            super::regions::callable(facts, definition, definition, source, function.signature)
                .map_err(error)?;
        let mut body = match default {
            LibraryDefault::OrdHelper { .. } => extrema::build(
                facts,
                definition,
                default,
                function.signature,
                source,
                &mut contract,
            ),
            LibraryDefault::PartialEqNe { eq } => {
                build(facts, definition, eq, function.signature, source)
            }
            LibraryDefault::PartialOrdRelation {
                partial_cmp,
                relation,
            } => ordering::build(
                facts,
                definition,
                partial_cmp,
                relation,
                function.signature,
                source,
            ),
        }
        .map_err(error)?;
        let query = crate::types::destruction::NeedsDrop::new(
            facts.destructors.implementations().iter().map(|i| i.item),
        );
        let mut query = query.scoped(Arc::clone(&environment));
        super::build::factory_owned_values(facts, &mut body, &mut query).map_err(error)?;
        let destruction = super::drops::requirements(facts, &body, &mut query).map_err(error)?;
        contracts.push(contract);
        templates.bodies.push(Template {
            body,
            destruction,
            environment,
        });
    }
    Ok(())
}
struct ProvidedCall {
    parameters: [TypeId; 2],
    result: TypeId,
    callee: TypeId,
    signature: TypeId,
}
fn call(
    facts: &mut Declarations,
    eq: DefId,
    signature: TypeId,
) -> Result<ProvidedCall, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        signature: declared,
        ..
    }) = facts
        .definitions
        .get(eq.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("provided comparison has no declared callee");
    };
    if item != crate::types::ItemId(eq.0) {
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
    let ([left, right], arguments) = operands(facts, parameters)?;
    if result != TypeId::primitive(crate::types::Primitive::Bool) {
        return Err("provided equality result is not bool");
    }
    let Some(Kind::Function {
        parameters: callee_parameters,
        result: called_result,
        bound_regions: 2,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    }) = facts.types.kind(declared)
    else {
        return Err("provided comparison callee has incorrect signature");
    };
    if parameters != callee_parameters
        || !matches!(
            facts.associated.comparison_method(eq),
            Some(
                crate::support::comparisons::Method::Eq
                    | crate::support::comparisons::Method::PartialCmp
            )
        )
        || facts.associated.comparison_method(eq) == Some(crate::support::comparisons::Method::Eq)
            && (called_result != result || declared != signature)
    {
        return Err("provided comparison callee differs from declared input/result contract");
    }
    let arguments = facts.types.intern_arguments(&arguments)?;
    let callable = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(eq.0),
        signature: declared,
        arguments,
    })?;
    let opened = facts.types.intern(Kind::Function {
        parameters,
        result: called_result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
        bound_regions: 0,
    })?;
    Ok(ProvidedCall {
        parameters: [left, right],
        result: called_result,
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
    let ProvidedCall {
        parameters: [left, right],
        result,
        callee: callable,
        signature: opened,
    } = call(facts, eq, signature)?;
    let mut body = Body::new(
        Owner::Function {
            definition,
            signature,
            execution: None,
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
            source: source.node.into(),
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
pub(super) fn statement(body: &mut Body, action: Action) {
    body.statements.push(Statement {
        source: body.root_site(),
        action,
    });
}
pub(super) fn value(body: &mut Body, ty: TypeId, kind: Rvalue) -> Result<ValueId, &'static str> {
    let id = ValueId(u32::try_from(body.values.len()).map_err(|_| "too many library values")?);
    body.values.push(Value { ty, kind });
    statement(body, Action::Evaluate(id));
    Ok(id)
}

fn operands(
    facts: &Declarations,
    parameters: crate::types::ListId,
) -> Result<([TypeId; 2], Vec<crate::types::arguments::Argument>), &'static str> {
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
    Ok(([left, right], arguments))
}

fn validate_default(
    facts: &Declarations,
    definition: DefId,
    default: LibraryDefault,
) -> Result<(), &'static str> {
    let validation = match default {
        LibraryDefault::OrdHelper { .. } => None,
        LibraryDefault::PartialEqNe { eq } => Some((
            eq,
            crate::support::comparisons::Method::Ne,
            crate::support::comparisons::Method::Eq,
        )),
        LibraryDefault::PartialOrdRelation {
            partial_cmp,
            relation,
        } => Some((
            partial_cmp,
            relation,
            crate::support::comparisons::Method::PartialCmp,
        )),
    };
    if let Some((callee, member, requirement)) = validation
        && (facts.associated.comparison_method(callee) != Some(requirement)
            || facts.associated.comparison_method(definition) != Some(member)
            || facts.associated.trait_owner(callee) != facts.associated.trait_owner(definition))
    {
        return Err("provided comparison body has a different canonical owner");
    }
    Ok(())
}
