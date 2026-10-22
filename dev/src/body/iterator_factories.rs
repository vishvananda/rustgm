//! Canonical iteration blankets have ordinary checked generic original bodies.
use super::{
    ir::{
        Action, Body, BorrowKind, Local, LocalId, LocalKind, Owner, Place, Projection,
        ProjectionId, ProjectionKind, Rvalue, ValueList,
    },
    provided::{statement, value},
    regions::{Contract, Kind as Debt, Obligation},
    templates::{Template, Templates},
};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{iteration::Original, model::Declarations},
    support::iteration::{Blanket, Trait},
    types::{ItemId, Kind, RegionId, arguments::Argument, substitute::Bound},
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../tests/invariants/iterator_contracts.rs"]
mod tests;

pub(crate) fn append(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut Templates,
    contracts: &mut Vec<Contract>,
) -> Result<(), Error> {
    let originals = facts
        .iteration
        .as_deref()
        .map_or([None; 2], |r| r.originals);
    for original in originals.into_iter().flatten() {
        let source = facts
            .iteration
            .as_deref()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing iteration original registry",
                )
            })?
            .canonical
            .origin;
        let error = |e| Declarations::error(report, source, e);
        if facts
            .iteration
            .as_deref()
            .and_then(|r| r.original(original.declaration.method))
            .is_none_or(|registered| {
                registered.signature != original.signature
                    || registered.predicate != original.predicate
                    || registered.parameter != original.parameter
            })
        {
            return Err(error(
                "iteration original differs from its registered factory record",
            ));
        }
        original.validate(facts).map_err(error)?;
        let environment = Arc::new(
            facts
                .generics
                .parameter_environment(&facts.types, original.declaration.method)
                .map_err(error)?,
        );
        let mut contract = super::regions::callable(
            facts,
            original.declaration.method,
            original.declaration.method,
            source,
            original.signature,
        )
        .map_err(error)?;
        let mut body = build(facts, original, &mut contract).map_err(error)?;
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
    super::collection_factories::append(report, facts, templates, contracts)?;
    super::clone_factories::append(report, facts, templates, contracts)?;
    super::range_factories::append(report, facts, templates, contracts)?;
    Ok(())
}
fn build(
    facts: &mut Declarations,
    original: Original,
    contract: &mut Contract,
) -> Result<Body, &'static str> {
    original.validate(facts)?;
    let source = facts
        .iteration
        .as_deref()
        .ok_or("missing iteration registry")?
        .canonical
        .origin;
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(original.signature)
    else {
        return Err("missing iteration original signature");
    };
    let [receiver] = facts
        .types
        .list(parameters)
        .ok_or("missing iteration original receiver")?
    else {
        return Err("incorrect iteration original arity");
    };
    let receiver = *receiver;
    facts.aggregates.prepare(&mut facts.types, result)?;
    let mut body = Body::new(
        Owner::Function {
            definition: original.declaration.method,
            signature: original.signature,
        },
        source,
        result,
    );
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        source.node,
    )));
    body.locals.push(Local {
        scope: super::scopes::Id(0),
        storage_parent: None,
        ty: receiver,
        kind: LocalKind::Parameter(0),
        definition: None,
        mutable: false,
        source: source.node,
    });
    statement(&mut body, Action::StorageLive(LocalId(0)));
    let returned = match original.declaration.kind {
        Blanket::Identity => value(&mut body, receiver, Rvalue::Read(Place::Local(LocalId(0))))?,
        Blanket::MutableReference => {
            forward(facts, original, contract, &mut body, receiver, result)?
        }
    };
    super::build::finish_factory(&mut body, returned)?;
    Ok(body)
}

fn forward(
    facts: &mut Declarations,
    original: Original,
    contract: &mut Contract,
    body: &mut Body,
    receiver: crate::types::TypeId,
    result: crate::types::TypeId,
) -> Result<super::ir::ValueId, &'static str> {
    let canonical = facts
        .iteration
        .as_deref()
        .ok_or("missing iteration registry")?
        .canonical;
    let source = canonical.origin;
    let place = mutable_referent(facts, body, receiver)?;
    let member = canonical.get(Trait::Iterator);
    let Some(Kind::FunctionItem { signature, .. }) =
        facts.definitions[member.method.0 as usize].and_then(|t| facts.types.kind(t))
    else {
        return Err("missing iteration forwarding callee");
    };
    let subject = facts.definitions[member.self_parameter.0 as usize]
        .ok_or("missing iteration Self parameter")?;
    let Some(Kind::Parameter(parameter)) = facts.types.kind(subject) else {
        return Err("invalid iteration Self parameter");
    };
    let mut fold =
        Bound::parameter_only(&facts.types, parameter, Argument::Type(original.parameter))?;
    let public = fold.apply(&mut facts.types, signature)?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(original.parameter)])?;
    let callable = facts.types.intern(Kind::FunctionItem {
        item: ItemId(member.method.0),
        signature: public,
        arguments,
    })?;
    let signature = Bound::new(vec![RegionId(1)]).signature(&mut facts.types, public, 0)?;
    let Some(Kind::Function {
        parameters,
        result: called_result,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("missing iteration forwarding signature");
    };
    let [borrow] = facts
        .types
        .list(parameters)
        .ok_or("missing iteration forwarding parameter")?
    else {
        return Err("invalid iteration forwarding arity");
    };
    let borrow = *borrow;
    let borrowed = value(
        body,
        borrow,
        Rvalue::Borrow {
            place,
            kind: BorrowKind::Immediate,
        },
    )?;
    let callee = value(body, callable, Rvalue::Function(DefId(member.method.0)))?;
    body.arguments.push(borrowed);
    contract.obligations.push(Obligation {
        source,
        kind: Debt::Variable {
            id: RegionId(1),
            needs_provenance: false,
        },
    });
    contract.obligations.push(Obligation {
        source,
        kind: Debt::LocalBorrow(RegionId(1)),
    });
    if called_result != result {
        return Err("iteration forwarding result differs from its declared Item");
    }
    value(
        body,
        result,
        Rvalue::Call {
            signature,
            callee,
            arguments: ValueList { start: 0, end: 1 },
        },
    )
}

fn mutable_referent(
    facts: &Declarations,
    body: &mut Body,
    receiver: crate::types::TypeId,
) -> Result<Place, &'static str> {
    let mut place = Place::Local(LocalId(0));
    let mut ty = receiver;
    for _ in 0..2 {
        let Some(Kind::Reference {
            referent,
            mutable: true,
            region,
        }) = facts.types.kind(ty)
        else {
            return Err("invalid iteration forwarding projection");
        };
        let id = ProjectionId(
            u32::try_from(body.projections.len()).map_err(|_| "too many iteration projections")?,
        );
        body.projections.push(Projection {
            base: place,
            kind: ProjectionKind::Dereference {
                region,
                mutable: true,
            },
            ty: referent,
        });
        statement(body, Action::Project(id));
        place = Place::Projection(id);
        ty = referent;
    }
    Ok(place)
}
