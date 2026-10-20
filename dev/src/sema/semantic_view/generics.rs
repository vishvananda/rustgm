//! PA21 inspects canonical generic contracts without changing caller permissions.
mod proofs;
use super::{Declarations, Report, io, relationships, type_view};
use std::io::Write;

/// Render typed generic relationships; region, loan and storage checks remain pending.
/// # Errors
/// Propagates output errors and inconsistent retained identities.
pub fn write(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "generic-semantics 1")?;
    super::records_with(report, facts, out, true)?;
    relationships::members(report, facts, out, true)?;
    relationships::constants(facts, out)?;
    relationships::normalized(facts, out)?;
    for (id, value) in facts.types.quantified_clauses().iter().enumerate() {
        write!(out, "quantifier {id} regions={} ", value.bound_regions)?;
        super::requirement(&facts.types, value.predicate, out)?;
        writeln!(out)?;
    }
    families(facts, out)?;
    opaques(facts, out)?;
    proofs::write(facts, out)?;
    write!(out, "eof ")?;
    count(report, facts, out)
}

/// Count actual retained facts without formatting, sorting or semantic queries.
/// # Errors
/// Propagates output errors.
pub fn count(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    relationships::totals(report, facts, out)?;
    write!(
        out,
        " quantifiers={} families={} family-bounds={} family-regions={} opaques={} opaque-bounds={} hidden-types={} opaque-body-results={} opaque-implementations={}",
        facts.types.quantified_clauses().len(),
        facts.associated.families.len(),
        facts
            .associated
            .families
            .values()
            .filter_map(|f| f.requirements.as_ref())
            .map(Vec::len)
            .sum::<usize>(),
        facts
            .associated
            .families
            .values()
            .map(|f| f.regions.len())
            .sum::<usize>(),
        facts.opaques.declarations.len(),
        facts
            .opaques
            .declarations
            .values()
            .map(|d| d.bounds.len())
            .sum::<usize>(),
        facts
            .opaques
            .declarations
            .values()
            .filter(|d| d.hidden.is_some())
            .count(),
        facts.opaques.body_results.len(),
        facts.opaques.implementations.len()
    )?;
    proofs::count(facts, out)?;
    writeln!(out)
}

fn families(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut families: Vec<_> = facts.associated.families.iter().collect();
    families.sort_unstable_by_key(|(id, _)| id.0);
    for (id, family) in families {
        writeln!(
            out,
            "family def={} captures={} regions={} complete={}",
            id.0,
            family.captures,
            family.total,
            family.requirements.is_some()
        )?;
        for (index, &bound) in family.requirements.iter().flatten().enumerate() {
            write!(out, "family-bound def={} index={index} ", id.0)?;
            super::clause(&facts.types, bound, out, true)?;
            writeln!(out)?;
        }
        for (index, &(longer, shorter)) in family.regions.iter().enumerate() {
            write!(out, "family-region def={} index={index} longer=", id.0)?;
            type_view::region(longer, out)?;
            write!(out, " shorter=")?;
            type_view::region(shorter, out)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

fn opaques(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    let mut declarations: Vec<_> = facts.opaques.declarations.iter().collect();
    declarations.sort_unstable_by_key(|(id, _)| id.0);
    for (id, declaration) in declarations {
        write!(
            out,
            "opaque {} owner={} source={}:{} application={} hidden=",
            id.0,
            declaration.owner.0,
            declaration.source.file.0,
            declaration.source.node.0,
            declaration.application.0
        )?;
        if let Some(hidden) = declaration.hidden {
            write!(out, "{}", hidden.0)?;
        } else {
            write!(out, "-")?;
        }
        let role = if facts
            .types
            .opaque_declaration(*id)
            .ok_or_else(|| io::Error::other("missing opaque type declaration"))?
            .trait_result
        {
            "trait-result"
        } else {
            "defining-result"
        };
        writeln!(out, " role={role} storage-proof=pending")?;
        for (index, &bound) in declaration.bounds.iter().enumerate() {
            write!(out, "opaque-bound id={} index={index} ", id.0)?;
            super::clause(&facts.types, bound, out, true)?;
            writeln!(out)?;
        }
    }
    let mut bodies: Vec<_> = facts.opaques.body_results.iter().collect();
    bodies.sort_unstable_by_key(|(id, _)| id.0);
    for (id, ty) in bodies {
        writeln!(out, "opaque-body-result def={} type={}", id.0, ty.0)?;
    }
    let mut implementations: Vec<_> = facts.opaques.implementations.iter().collect();
    implementations.sort_unstable_by_key(|((owner, id), _)| (owner.0, id.0));
    for ((owner, id), result) in implementations {
        write!(
            out,
            "opaque-implementation owner={} id={} type={} parent-regions={} regions=[",
            owner.0, id.0, result.ty.0, result.parent_regions
        )?;
        for (index, &region) in result.regions.iter().enumerate() {
            if index != 0 {
                write!(out, ",")?;
            }
            type_view::region(region, out)?;
        }
        writeln!(out, "]")?;
    }
    Ok(())
}
