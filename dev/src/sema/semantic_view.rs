//! Deterministic associated-item inspection consumes completed typed facts.
pub mod relationships;
use super::{associated::OwnerKind, model::Declarations, type_view};
use crate::{
    resolve::{expansion::Report, model::Target, unit::NodeKey},
    support::quoted,
    types::{
        Store,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::io::{self, Write};

/// Inspect type and trait decisions without claiming completed region or loan proofs.
/// # Errors
/// Propagates output errors and inconsistent retained identities.
pub fn write(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "semantic-crate 1")?;
    records(report, facts, out)?;
    write!(out, "eof ")?;
    count(facts, out)
}

fn records(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    type_view::types(&facts.types, out)?;
    type_view::definitions(report, facts, out)?;
    owners(report, facts, out)?;
    parameters(facts, out)?;
    predicates(facts, out)?;
    implementations(facts, out)?;
    uses(facts, out)?;
    type_view::body_regions(&facts.body_regions, out)?;
    type_view::regions(&facts.region_equalities, out)?;
    type_view::coercions(&facts.coercions, out)?;
    type_view::outlives(&facts.outlives, out)?;
    for bound in &facts.type_outlives {
        write!(
            out,
            "type-outlives body={}:{} source={}:{} type={} shorter=",
            bound.body.file.0,
            bound.body.node.0,
            bound.source.file.0,
            bound.source.node.0,
            bound.ty.0
        )?;
        type_view::region(bound.shorter, out)?;
        writeln!(out, " pending")?;
    }
    Ok(())
}

/// Count the same retained facts without rendering their text or rerunning inference.
/// # Errors
/// Propagates output errors.
pub fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    totals(facts, out)?;
    writeln!(out)
}

fn totals(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "owners={} members={} parameters={} predicates={} impls={} calls={} paths={} impl-uses={}",
        facts.associated.owners().len(),
        facts.associated.members().len(),
        facts
            .generics
            .parameters
            .values()
            .map(Vec::len)
            .sum::<usize>(),
        facts
            .generics
            .predicates
            .values()
            .map(Vec::len)
            .sum::<usize>(),
        facts.generics.implementations().count(),
        facts.methods.calls.len(),
        facts.methods.qualified.len() + facts.generics.trait_uses.len(),
        facts
            .generics
            .trait_selections
            .values()
            .map(Vec::len)
            .sum::<usize>()
    )
}

fn owners(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for (index, owner) in facts.associated.owners().iter().enumerate() {
        let kind = match owner.kind {
            OwnerKind::Trait { .. } => "trait",
            OwnerKind::Impl {
                trait_type: None, ..
            } => "inherent",
            OwnerKind::Impl {
                trait_type: Some(_),
                ..
            } => "trait-impl",
        };
        let ty = facts
            .associated
            .schema(owner.self_definition)
            .map(|s| s.self_type)
            .or_else(|| {
                facts
                    .destructors
                    .self_type(owner.self_definition)
                    .map(|i| i.target)
            })
            .ok_or_else(|| io::Error::other("missing associated Self type"))?;
        write!(
            out,
            "associated-owner {index} kind={kind} self-def={} self={} source={}:{}",
            owner.self_definition.0, ty.0, owner.source.file.0, owner.source.node.0
        )?;
        if let OwnerKind::Trait {
            definition,
            unsafe_,
        } = owner.kind
        {
            write!(out, " trait={} unsafe={unsafe_} name=", definition.0)?;
            let definition = &report.resolution.definitions[definition.0 as usize];
            quoted::bytes(
                out,
                report.package.names.get(definition.name).as_bytes(),
                false,
            )?;
        } else if let OwnerKind::Impl { unsafe_, .. } = owner.kind {
            write!(out, " unsafe={unsafe_}")?;
        }
        writeln!(out)?;
    }
    for member in facts.associated.members() {
        write!(
            out,
            "associated-member def={} owner={}",
            member.definition.0, member.owner.0
        )?;
        if let Some((ty, depth)) = facts.methods.receivers.get(&member.definition) {
            write!(out, " receiver={} references={depth}", ty.0)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn parameters(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut owners: Vec<_> = facts.generics.parameters.iter().collect();
    owners.sort_unstable_by_key(|(id, _)| id.0);
    for (owner, parameters) in owners {
        for parameter in parameters {
            write!(
                out,
                "parameter owner={} binding={} argument=",
                owner.0, parameter.binding.0
            )?;
            type_view::argument(&facts.types, parameter.argument, out)?;
            write!(out, " default=")?;
            if let Some(default) = parameter.default {
                type_view::argument(&facts.types, default, out)?;
            } else {
                write!(out, "-")?;
            }
            writeln!(out)?;
        }
    }
    Ok(())
}

fn predicates(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut owners: Vec<_> = facts.generics.predicates.iter().collect();
    owners.sort_unstable_by_key(|(id, _)| id.0);
    for (owner, predicates) in owners {
        for predicate in predicates {
            write!(
                out,
                "predicate owner={} source={}:{} ",
                owner.0, predicate.source.file.0, predicate.source.node.0
            )?;
            requirement(
                &facts.types,
                predicate.value.ordinary().map_err(io::Error::other)?,
                out,
            )?;
            writeln!(out)?;
        }
    }
    Ok(())
}

fn requirement(store: &Store, predicate: Predicate, out: &mut impl Write) -> io::Result<()> {
    write!(out, "subject={} bound=", predicate.ty.0)?;
    match predicate.bound {
        Bound::ProjectionEqual(result) => write!(out, "projection-equal:{}", result.0),
        Bound::Sized => write!(out, "sized"),
        Bound::Copy => write!(out, "copy"),
        Bound::Clone => write!(out, "clone"),
        Bound::Outlives(region) => {
            write!(out, "outlives:")?;
            type_view::region(region, out)
        }
        Bound::Trait(id) => {
            let application = store
                .trait_application(id)
                .ok_or_else(|| io::Error::other("invalid trait application"))?;
            write!(out, "trait:{}", application.item.0)?;
            type_view::generic_arguments(store, application.arguments, out)
        }
    }
}

fn implementations(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for implementation in facts.generics.implementations() {
        write!(
            out,
            "impl owner={} source={}:{} ",
            implementation.owner.0, implementation.source.file.0, implementation.source.node.0
        )?;
        requirement(&facts.types, implementation.predicate, out)?;
        writeln!(out)?;
        for &(declaration, selected) in &implementation.methods {
            writeln!(
                out,
                "impl-method owner={} declaration={} selected={} default={}",
                implementation.owner.0,
                declaration.0,
                selected.unwrap_or(declaration).0,
                selected.is_none()
            )?;
        }
        for &(declaration, definition) in &implementation.types {
            writeln!(
                out,
                "impl-type owner={} declaration={} definition={}",
                implementation.owner.0, declaration.0, definition.0
            )?;
        }
    }
    Ok(())
}

fn uses(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut calls: Vec<_> = facts.methods.calls.iter().collect();
    calls.sort_unstable_by_key(|(key, _)| source(**key));
    for (key, call) in calls {
        let origin = if facts.associated.trait_owner(call.method).is_none() {
            "inherent"
        } else if facts.methods.bound_uses.contains(key) {
            "bound"
        } else {
            "lexical"
        };
        write!(
            out,
            "method-call source={}:{} method={} callable={} dereferences={} borrow=",
            key.file.0, key.node.0, call.method.0, call.callable.0, call.dereferences
        )?;
        if let Some(borrow) = call.borrow {
            write!(out, "{}", borrow.0)?;
        } else {
            write!(out, "-")?;
        }
        writeln!(out, " receiver={} origin={origin}", call.receiver.0)?;
    }
    paths(facts, out)?;
    let mut selections: Vec<_> = facts.generics.trait_selections.iter().collect();
    selections.sort_unstable_by_key(|(key, _)| source(**key));
    for (key, selections) in selections {
        for selection in selections {
            write!(
                out,
                "implementation-use source={}:{} owner={}",
                key.file.0, key.node.0, selection.owner.0
            )?;
            type_view::generic_arguments(&facts.types, selection.arguments, out)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

fn paths(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut inherent: Vec<_> = facts.methods.qualified.iter().collect();
    inherent.sort_unstable_by_key(|(key, _)| source(**key));
    for (key, path) in inherent {
        writeln!(
            out,
            "inherent-path source={}:{} method={} implementation={} prefix={}:{} annotation={}",
            key.file.0,
            key.node.0,
            selected_method(facts, *key)?.0,
            path.implementation.0,
            path.prefix.file.0,
            path.prefix.node.0,
            path.annotation
        )?;
    }
    let mut traits: Vec<_> = facts.generics.trait_uses.iter().collect();
    traits.sort_unstable_by_key(|(key, _)| source(**key));
    for (key, path) in traits {
        let qualifier = if path.arguments.is_some() {
            "explicit"
        } else {
            "implicit"
        };
        let origin = if facts.methods.bound_uses.contains(key) {
            "bound"
        } else {
            "ordinary"
        };
        write!(
            out,
            "trait-path source={}:{} method={} trait={} qualifier={qualifier} origin={origin} subject-source=",
            key.file.0,
            key.node.0,
            selected_method(facts, *key)?.0,
            path.owner.0
        )?;
        if let Some(subject) = path.subject {
            write!(out, "{}:{}", subject.file.0, subject.node.0)?;
        } else {
            write!(out, "-")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn selected_method(facts: &Declarations, key: NodeKey) -> io::Result<crate::resolve::model::DefId> {
    match facts.node(key).and_then(|n| n.reference) {
        Some(Target::Definition(id)) => Ok(id),
        _ => Err(io::Error::other("selected method path has no declaration")),
    }
}
fn source(key: NodeKey) -> (u32, u32) {
    (key.file.0, key.node.0)
}
