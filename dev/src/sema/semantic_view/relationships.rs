//! PA20 inspects retained associated contracts and normal forms without new semantic queries.
use super::{Declarations, Report, io, type_view};
use crate::resolve::model::DefKind;
use std::io::Write;

/// Render completed trait relationships while leaving later lifetime/loan proof pending.
/// # Errors
/// Propagates output failures and inconsistent retained declaration identities.
pub fn write(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "trait-semantics 1")?;
    super::records(report, facts, out)?;
    members(report, facts, out)?;
    constants(facts, out)?;
    normalized(facts, out)?;
    write!(out, "eof ")?;
    count(report, facts, out)
}

/// Count exactly the emitted retained facts, without sorting or running selection.
/// # Errors
/// Propagates output failures.
pub fn count(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut types = 0;
    let mut constants = 0;
    let mut contracts = 0;
    let mut bounds = 0;
    for member in facts.associated.members() {
        match report.resolution.definitions[member.definition.0 as usize].kind {
            DefKind::TypeAlias => types += 1,
            DefKind::Const => constants += 1,
            _ => {}
        }
        if let Some(contract) = facts.associated.type_contract(member.definition) {
            contracts += 1;
            bounds += contract.bounds.len();
        }
    }
    let (nodes, definitions) = facts.unreduced.normalization_counts();
    super::totals(facts, out)?;
    writeln!(
        out,
        " type-members={types} constant-members={constants} result-contracts={contracts} result-bounds={bounds} impl-constants={} constant-uses={} constant-dispatches={} normalized-nodes={nodes} normalized-definitions={definitions}",
        facts
            .generics
            .implementations()
            .map(|value| value.constants.len())
            .sum::<usize>(),
        facts.constant_uses.len(),
        facts.constant_targets.len(),
    )
}

fn members(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for member in facts.associated.members() {
        let label = match report.resolution.definitions[member.definition.0 as usize].kind {
            DefKind::TypeAlias => "associated-type",
            DefKind::Const => "associated-constant",
            _ => continue,
        };
        let ty = facts
            .definitions
            .get(member.definition.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| io::Error::other("associated member has no retained type"))?;
        writeln!(out, "{label} def={} type={}", member.definition.0, ty.0)?;
        if let Some(contract) = facts.associated.type_contract(member.definition) {
            writeln!(
                out,
                "result-contract def={} projection={}",
                member.definition.0, contract.projection.0
            )?;
            for (index, &bound) in contract.bounds.iter().enumerate() {
                write!(
                    out,
                    "result-bound def={} index={index} ",
                    member.definition.0
                )?;
                super::requirement(
                    &facts.types,
                    bound.ordinary().map_err(io::Error::other)?,
                    out,
                )?;
                writeln!(out)?;
            }
        }
    }
    Ok(())
}

fn constants(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for implementation in facts.generics.implementations() {
        for &(declaration, selected) in &implementation.constants {
            writeln!(
                out,
                "impl-constant owner={} declaration={} selected={} default={}",
                implementation.owner.0,
                declaration.0,
                selected.unwrap_or(declaration).0,
                selected.is_none()
            )?;
        }
    }
    let mut uses: Vec<_> = facts.constant_uses.iter().collect();
    uses.sort_unstable_by_key(|(key, _)| super::source(**key));
    for (key, instance) in uses {
        write!(
            out,
            "constant-use source={}:{} definition={}",
            key.file.0, key.node.0, instance.definition.0
        )?;
        type_view::generic_arguments(&facts.types, instance.arguments, out)?;
        writeln!(out)?;
    }
    let mut dispatches: Vec<_> = facts.constant_targets.iter().collect();
    dispatches.sort_unstable_by_key(|(instance, _)| (instance.definition.0, instance.arguments.0));
    for (instance, target) in dispatches {
        write!(
            out,
            "constant-dispatch declaration={}",
            instance.definition.0
        )?;
        type_view::generic_arguments(&facts.types, instance.arguments, out)?;
        write!(out, " selected={}", target.definition.0)?;
        type_view::generic_arguments(&facts.types, target.arguments, out)?;
        writeln!(out)?;
    }
    Ok(())
}

fn normalized(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut nodes: Vec<_> = facts.unreduced.node_entries().collect();
    nodes.sort_unstable_by_key(|&(key, _)| super::source(key));
    for (key, original) in nodes {
        let result = facts
            .node(key)
            .and_then(|node| node.ty)
            .ok_or_else(|| io::Error::other("normalized source has no retained result"))?;
        writeln!(
            out,
            "normalized-node source={}:{} original={} result={}",
            key.file.0, key.node.0, original.0, result.0
        )?;
    }
    let mut definitions: Vec<_> = facts.unreduced.definition_entries().collect();
    definitions.sort_unstable_by_key(|&(id, _)| id.0);
    for (id, original) in definitions {
        let result = facts
            .definitions
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| io::Error::other("normalized declaration has no retained result"))?;
        writeln!(
            out,
            "normalized-definition def={} original={} result={}",
            id.0, original.0, result.0
        )?;
    }
    Ok(())
}
