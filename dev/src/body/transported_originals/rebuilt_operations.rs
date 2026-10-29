//! Reconstruct the admitted source operation domain from its sealed current view.
use super::{
    ImportedOriginals, encoded_literals,
    operations::{self, lexical},
};
use crate::{
    body::{
        drops::values,
        ir::{Action, Body, LocalId, Owner, Place, Rvalue, Statement, Value, ValueId},
    },
    sema::crates::SourceOriginal,
    support::crates::generics::source as s,
};

#[cfg(test)]
#[path = "../../../tests/invariants/effect_artifacts.rs"]
mod effect_tests;

pub(super) fn build(
    view: &SourceOriginal<'_>,
    work: &mut ImportedOriginals,
) -> Result<Body, &'static str> {
    let record = view.raw();
    if (!record.variables.is_empty() && !view.numbered())
        || !record.projections.is_empty()
        || (!v4(view) && (!record.calls.is_empty() || !record.arguments.is_empty()))
        || !record.openings.is_empty()
    {
        return Err(
            "source regional/projected/call body requires its wider ordinary import profile",
        );
    }
    work.charge(
        record
            .locals
            .len()
            .saturating_add(record.values.len())
            .saturating_add(record.statements.len())
            .saturating_add(record.blocks.len())
            .saturating_add(record.scopes.len())
            .saturating_add(record.exits.len())
            .saturating_add(record.temporaries.len())
            .saturating_add(1),
    )?;
    let mut body = Body::original(
        Owner::Function {
            definition: view.definition(),
            signature: view.signature(),
            execution: None,
        },
        view.source()?,
        view.ty(record.result, &mut work.steps)?,
    );
    body.const_context = view.const_context();
    body.locals = operations::locals(&record.locals, |local| {
        Ok(operations::LocalData {
            ty: view.ty(local.ty, &mut work.steps)?,
            definition: local
                .definition
                .map(|d| view.declaration(d, &mut work.steps))
                .transpose()?,
            source: view.site(local.site, &mut work.steps)?,
        })
    })?;
    encoded_literals::allocate(record, &mut body, work)?;
    body.values = record.values.iter().map(|value| Ok(Value {
        ty: view.ty(value.ty, &mut work.steps)?,
        kind: match value.kind {
            s::ValueKind::Unit => Rvalue::Unit,
            s::ValueKind::Scalar(_) => encoded_literals::append(view, *value, &mut body)?,
            s::ValueKind::Read(place) => Rvalue::Read(local(place)?),
            s::ValueKind::Constrain(value) => Rvalue::Constrain(ValueId(value)),
            s::ValueKind::Function(application) if v4(view) => Rvalue::Function(view.function(application, &mut work.steps)?),
            s::ValueKind::Call {signature, callee, arguments} if v4(view) => Rvalue::Call {
                signature:view.ty(signature, &mut work.steps)?, callee:ValueId(callee),
                arguments:crate::body::ir::ValueList {start:arguments.start, end:arguments.end},
            },
            _ => return Err("source value operation requires its wider actual reconstruction profile"),
        },
    })).collect::<Result<_, &'static str>>()?;
    work.charge(record.arguments.len())?;
    body.arguments = record.arguments.iter().copied().map(ValueId).collect();
    body.statements = record
        .statements
        .iter()
        .map(|statement| {
            Ok(Statement {
                source: view.site(statement.site, &mut work.steps)?,
                action: action(statement.action)?,
            })
        })
        .collect::<Result<_, &'static str>>()?;
    body.blocks = operations::blocks(&record.blocks, &mut |index| {
        view.site(index, &mut work.steps)
    })?;
    lexical::fill_records(
        lexical::Records {
            scopes: &record.scopes,
            exits: &record.exits,
            temporaries: &record.temporaries,
        },
        &mut body,
        &mut |index| view.site(index, &mut work.steps),
    )?;
    owned(view, &mut body, work)?;
    Ok(body)
}
fn local(place: s::Place) -> Result<Place, &'static str> {
    match place {
        s::Place::Local(local) => Ok(Place::Local(LocalId(local))),
        s::Place::Projection(_) => Err("source projection requires its wider actual place profile"),
    }
}
fn action(record: s::Action) -> Result<Action, &'static str> {
    Ok(match record {
        s::Action::StorageLive(id) => Action::StorageLive(LocalId(id)),
        s::Action::StorageDead(id) => Action::StorageDead(LocalId(id)),
        s::Action::StorageEnd { first, last } => Action::StorageEnd {
            first: LocalId(first),
            last: LocalId(last),
        },
        s::Action::Evaluate(id) => Action::Evaluate(ValueId(id)),
        s::Action::Store { place, value } => Action::Store {
            place: local(place)?,
            value: ValueId(value),
        },
        s::Action::CheckInitialized(place) => Action::CheckInitialized(local(place)?),
        s::Action::Mention(place) => Action::Mention(local(place)?),
        s::Action::Project(_) => {
            return Err("source projection statement requires its wider actual profile");
        }
    })
}
fn owned(
    view: &SourceOriginal<'_>,
    body: &mut Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let Some(record) = &view.raw().owned else {
        return Ok(());
    };
    let mut owned = values::Facts::default();
    for creation in &record.created {
        work.charge(1)?;
        owned.insert(lexical::temporary(creation, &mut |index| {
            view.site(index, &mut work.steps)
        })?)?;
    }
    for release in &record.returned {
        work.charge(1)?;
        owned.returned.push(values::Release {
            value: ValueId(release.value),
            location: lexical::location(release.location),
            exits_before: release.exits_before,
        });
    }
    body.owned = Some(Box::new(owned));
    Ok(())
}
pub(super) fn allocation(
    record: &s::Original,
    work: &mut ImportedOriginals,
) -> Result<usize, &'static str> {
    use crate::body::{ir, scopes, storage};
    Ok(encoded_literals::count(record, work)?
        .saturating_mul(size_of::<crate::sema::model::Scalar>())
        .saturating_add(
            record
                .locals
                .len()
                .saturating_mul(size_of::<ir::Local>())
                .saturating_add(record.values.len().saturating_mul(size_of::<ir::Value>()))
                .saturating_add(
                    record
                        .statements
                        .len()
                        .saturating_mul(size_of::<ir::Statement>()),
                )
                .saturating_add(record.blocks.len().saturating_mul(size_of::<ir::Block>()))
                .saturating_add(
                    record
                        .scopes
                        .len()
                        .saturating_mul(size_of::<storage::Scope>()),
                )
                .saturating_add(
                    record
                        .exits
                        .len()
                        .saturating_mul(size_of::<storage::Exit>()),
                )
                .saturating_add(
                    record
                        .temporaries
                        .len()
                        .saturating_mul(size_of::<ir::Temporary>()),
                )
                .saturating_add(record.owned.as_ref().map_or(0, |owned| {
                    size_of::<values::Facts>()
                        .saturating_add(
                            owned
                                .created
                                .len()
                                .saturating_mul(size_of::<ir::Temporary>()),
                        )
                        .saturating_add(
                            owned
                                .returned
                                .len()
                                .saturating_mul(size_of::<values::Release>()),
                        )
                        .saturating_add(crate::support::retained::new_map::<ir::ValueId, usize>(
                            owned.created.len(),
                        ))
                }))
                // Keep alignment overhead conservative without flattening any owner graph.
                .saturating_add(
                    record
                        .arguments
                        .len()
                        .saturating_mul(size_of::<ir::ValueId>()),
                )
                .saturating_add(
                    size_of::<Body>() + size_of::<storage::Facts>() + size_of::<scopes::Id>(),
                ),
        ))
}

pub(super) fn validate(
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let record = view.raw();
    if body.source != view.source()?
        || body.const_context != view.const_context()
        || body.instance.is_some()
        || !matches!(body.owner, Owner::Function { definition, signature, execution: None } if definition == view.definition() && signature == view.signature())
        || body.result != view.ty(record.result, steps)?
        || body.locals.len() != record.locals.len()
        || body.values.len() != record.values.len()
        || body.statements.len() != record.statements.len()
        || body.blocks.len() != record.blocks.len()
        || body.temporaries.len() != record.temporaries.len()
        || body.static_types.is_some()
        || body.mutable_statics.is_some()
        || body.resolved_callables.is_some()
        || body.closure_calls.is_some()
        || !body.guards.is_empty()
        || !body.guard_places.is_empty()
        || !body.projections.is_empty()
        || body.arguments.len() != record.arguments.len()
        || body
            .arguments
            .iter()
            .zip(&record.arguments)
            .any(|(actual, raw)| actual.0 != *raw)
        || body
            .promotions
            .as_ref()
            .is_some_and(|facts| !facts.is_empty())
    {
        return Err("source original changed its complete supported body/header domain");
    }
    ports(view, body, steps)?;
    for (actual, record) in body.statements.iter().zip(&record.statements) {
        *steps = steps.saturating_add(1);
        if actual.source != view.site(record.site, steps)?
            || !same_action(actual.action, action(record.action)?)
        {
            return Err("source original changed an actual statement/site");
        }
    }
    for (actual, record) in body.blocks.iter().zip(&record.blocks) {
        use crate::{body::ir::Terminator as T, support::crates::generics::Terminator as R};
        *steps = steps.saturating_add(1);
        let same = match (actual.terminator, record.terminator) {
            (T::Return(a), R::Return(b)) => a.0 == b,
            (T::Jump(a), R::Jump(b)) => a.0 == b,
            (T::Unreachable, R::Unreachable) => true,
            _ => false,
        };
        if !same
            || actual.start != record.statements.start
            || actual.end != record.statements.end
            || actual.source != view.site(record.site, steps)?
        {
            return Err("source original changed an actual CFG block/edge");
        }
    }
    storage(view, body, steps)?;
    for (actual, record) in body.temporaries.iter().zip(&record.temporaries) {
        let expected = lexical::temporary(record, &mut |index| view.site(index, steps))?;
        if !same_temporary(actual, &expected) {
            return Err("source original changed a scoped temporary");
        }
    }
    validate_owned(view, body, steps)
}
fn ports(view: &SourceOriginal<'_>, body: &Body, steps: &mut usize) -> Result<(), &'static str> {
    let record = view.raw();
    for (actual, record) in body.locals.iter().zip(&record.locals) {
        *steps = steps.saturating_add(1);
        let kind = match record.kind {
            crate::support::crates::generics::LocalKind::Parameter(index) => {
                crate::body::ir::LocalKind::Parameter(index)
            }
            crate::support::crates::generics::LocalKind::Binding => {
                crate::body::ir::LocalKind::Binding
            }
            crate::support::crates::generics::LocalKind::Merge => crate::body::ir::LocalKind::Merge,
        };
        if actual.ty != view.ty(record.ty, steps)?
            || actual.source != view.site(record.site, steps)?
            || actual.kind != kind
            || actual.scope.0 != record.scope
            || actual.storage_parent.map(|id| id.0) != record.storage_parent
            || actual.definition
                != record
                    .definition
                    .map(|id| view.declaration(id, steps))
                    .transpose()?
            || actual.mutable != record.mutable
        {
            return Err("source original changed an actual typed local/binding/scope");
        }
    }
    let mut scalar = 0;
    for (actual, record) in body.values.iter().zip(&record.values) {
        *steps = steps.saturating_add(1);
        let same = match (actual.kind, record.kind) {
            (Rvalue::Unit, s::ValueKind::Unit) => true,
            (Rvalue::Scalar(id), s::ValueKind::Scalar(_)) => {
                encoded_literals::same(view, *record, id, body, &mut scalar)?
            }
            (Rvalue::Read(actual), s::ValueKind::Read(record)) => actual == local(record)?,
            (Rvalue::Constrain(actual), s::ValueKind::Constrain(record)) => actual.0 == record,
            (Rvalue::Function(actual), s::ValueKind::Function(raw)) if v4(view) => {
                actual == view.function(raw, steps)?
            }
            (
                Rvalue::Call {
                    signature,
                    callee,
                    arguments,
                },
                s::ValueKind::Call {
                    signature: raw,
                    callee: c,
                    arguments: a,
                },
            ) if v4(view) => {
                signature == view.ty(raw, steps)?
                    && callee.0 == c
                    && arguments.start == a.start
                    && arguments.end == a.end
            }
            _ => false,
        };
        if !same || actual.ty != view.ty(record.ty, steps)? {
            return Err("source original changed an actual typed value operation");
        }
    }
    if scalar != body.scalars.len() {
        return Err("source original changed its complete scalar storage domain");
    }
    Ok(())
}
fn same_action(a: Action, b: Action) -> bool {
    match (a, b) {
        (Action::StorageLive(a), Action::StorageLive(b))
        | (Action::StorageDead(a), Action::StorageDead(b)) => a == b,
        (Action::StorageEnd { first: a, last: b }, Action::StorageEnd { first: x, last: y }) => {
            a == x && b == y
        }
        (Action::Evaluate(a), Action::Evaluate(b)) => a == b,
        (Action::Store { place: a, value: b }, Action::Store { place: x, value: y }) => {
            a == x && b == y
        }
        (Action::CheckInitialized(a), Action::CheckInitialized(b))
        | (Action::Mention(a), Action::Mention(b)) => a == b,
        _ => false,
    }
}
fn storage(view: &SourceOriginal<'_>, body: &Body, steps: &mut usize) -> Result<(), &'static str> {
    use crate::{
        body::{scopes, storage},
        support::crates::generics as g,
    };
    let actual = body
        .storage
        .as_ref()
        .ok_or("source original omitted its actual storage boundaries")?;
    let record = view.raw();
    if actual.scopes.len() != record.scopes.len() || actual.exits.len() != record.exits.len() {
        return Err("source original changed its complete lexical/storage domain");
    }
    for (actual, record) in actual.scopes.iter().zip(&record.scopes) {
        *steps = steps.saturating_add(1);
        if actual.parent.map(|id| id.0) != record.parent
            || actual.source != view.site(record.site, steps)?
            || actual.kind != lexical::scope_kind(record.kind)
        {
            return Err("source original changed a lexical scope");
        }
    }
    for (actual, record) in actual.exits.iter().zip(&record.exits) {
        *steps = steps.saturating_add(1);
        let end = match record.end {
            g::End::Scopes { inner, outer } => storage::End::Scopes(storage::Range {
                inner: scopes::Id(inner),
                outer: outer.map(scopes::Id),
            }),
            g::End::Temporaries(r) => storage::End::Temporaries {
                start: r.start,
                end: r.end,
            },
        };
        if actual.location != lexical::location(record.location) || actual.end != end {
            return Err("source original changed a storage exit/order");
        }
    }
    Ok(())
}
fn validate_owned(
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    match (&view.raw().owned, &body.owned) {
        (None, None) => Ok(()),
        (Some(record), Some(actual)) => {
            if record.created.len() != actual.created.len()
                || record.returned.len() != actual.returned.len()
                || !actual.has_exact_index(steps)
            {
                return Err("source original changed its complete owned-value coverage");
            }
            for (record, actual) in record.created.iter().zip(&actual.created) {
                *steps = steps.saturating_add(1);
                let expected = lexical::temporary(record, &mut |index| view.site(index, steps))?;
                if !same_temporary(actual, &expected) {
                    return Err("source original changed an owned-value creation");
                }
            }
            for (record, actual) in record.returned.iter().zip(&actual.returned) {
                *steps = steps.saturating_add(1);
                if actual.value.0 != record.value
                    || actual.location != lexical::location(record.location)
                    || actual.exits_before != record.exits_before
                {
                    return Err("source original changed an owned-value transfer/exit order");
                }
            }
            Ok(())
        }
        _ => Err("source original changed absent-versus-present owned-value records"),
    }
}
fn same_temporary(a: &crate::body::ir::Temporary, b: &crate::body::ir::Temporary) -> bool {
    a.lifetime == b.lifetime
        && a.value == b.value
        && a.source == b.source
        && a.block == b.block
        && a.before == b.before
        && a.exits_before == b.exits_before
}

fn v4(view: &SourceOriginal<'_>) -> bool {
    view.rows().descriptor().profile().feature_predicates()
}
