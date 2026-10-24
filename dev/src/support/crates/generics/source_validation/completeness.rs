//! Header ownership and complete clause/body coverage are indexed once.
#[path = "header_ledger/supplied_bodies.rs"]
mod body_coverage;
#[path = "header_ledger/formal_rows.rs"]
mod formal_rows;
#[path = "header_ledger/membership_rows.rs"]
mod member_rows;
use super::{DefinitionKind, HeaderRef, SourceRows, Work, declaration};
use crate::support::crates::{
    generics::{self as g, source as s},
    owner_catalog::DeclarationRef,
};

pub(super) fn validate(
    rows: &SourceRows,
    work: &mut Work,
) -> Result<Vec<Option<HeaderRef>>, &'static str> {
    work.charge(
        rows.declarations
            .len()
            .saturating_add(rows.clauses.len().saturating_mul(2)),
    )?;
    let mut headers = vec![None; rows.declarations.len()];
    let mut clauses = vec![false; rows.clauses.len()];
    let mut literal_inputs = vec![false; rows.clauses.len()];
    for (index, header) in rows.traits.iter().enumerate() {
        trait_header(
            rows,
            header,
            index,
            &mut headers,
            &mut clauses,
            &mut literal_inputs,
            work,
        )?;
    }
    for (index, header) in rows.implementations.iter().enumerate() {
        insert(
            rows,
            &mut headers,
            header.self_definition,
            DefinitionKind::SelfType,
            HeaderRef::Implementation(ordinal(index)?),
            work,
        )?;
        if header.scope != header.self_definition {
            return Err("source implementation scope differs from its genuine Self owner");
        }
        inputs(
            rows,
            header.self_definition,
            header.inputs,
            header.declared_inputs,
            None,
            &mut clauses,
            work,
        )?;
    }
    for (index, header) in rows.functions.iter().enumerate() {
        insert(
            rows,
            &mut headers,
            header.declaration,
            DefinitionKind::Function,
            HeaderRef::Function(ordinal(index)?),
            work,
        )?;
        if header.scope != header.declaration {
            return Err("source function scope differs from its actual declaration");
        }
        inputs(
            rows,
            header.declaration,
            header.inputs,
            header.declared_inputs,
            None,
            &mut clauses,
            work,
        )?;
    }
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        if matches!(
            row.definition.kind,
            DefinitionKind::Function | DefinitionKind::Trait
        ) && headers[index].is_none()
        {
            return Err("source declaration has no complete typed header");
        }
        if row.definition.kind == DefinitionKind::SelfType && headers[index].is_none() {
            let parent = row
                .definition
                .parent
                .and_then(|parent| headers.get(parent.0 as usize));
            let Some(Some(HeaderRef::Trait(trait_))) = parent else {
                return Err(
                    "source Self declaration belongs to neither a trait nor an implementation",
                );
            };
            if rows.traits[*trait_ as usize].self_definition.0 as usize != index {
                return Err("source trait has an extra foreign Self declaration");
            }
        }
    }
    work.charge(clauses.len())?;
    if clauses.iter().any(|used| !used) {
        return Err("source clauses have incomplete owning header coverage");
    }
    member_rows::validate(rows, &headers, work)?;
    body_coverage::validate(rows, &headers, work)?;
    formal_rows::validate(rows, work)?;
    Ok(headers)
}

fn trait_header(
    rows: &SourceRows,
    header: &s::TraitHeader,
    index: usize,
    headers: &mut [Option<HeaderRef>],
    clauses: &mut [bool],
    literals: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    insert(
        rows,
        headers,
        header.declaration,
        DefinitionKind::Trait,
        HeaderRef::Trait(ordinal(index)?),
        work,
    )?;
    declaration(rows, header.self_definition, DefinitionKind::SelfType, work)?;
    if header.scope != header.declaration
        || rows.declarations[header.self_definition.0 as usize]
            .definition
            .parent
            != Some(header.declaration)
        || header.canonical_inputs >= header.declared_inputs
    {
        return Err("source trait lost its actual Self owner or canonical prefix boundary");
    }
    inputs(
        rows,
        header.declaration,
        header.inputs,
        header.declared_inputs,
        Some(header.canonical_inputs),
        clauses,
        work,
    )?;
    let literal = header.written_supertraits.slice(&rows.clause_indices)?;
    for &input in literal {
        work.charge(1)?;
        if input < header.inputs.start
            || input >= header.inputs.start.saturating_add(header.canonical_inputs)
            || rows.clauses[input as usize].self_subject.is_none()
            || std::mem::replace(&mut literals[input as usize], true)
        {
            return Err("source literal supertrait index is foreign, repeated or noncanonical");
        }
    }
    let implicit = &rows.clauses[(header.inputs.start + header.canonical_inputs) as usize];
    let s::Requirement::Trait {
        application,
        effect: crate::support::constness::Constness::Ordinary,
    } = implicit.requirement
    else {
        return Err("source trait implicit Self input has a different qualified kind");
    };
    let application = rows
        .trait_applications
        .get(application.0 as usize)
        .ok_or("source implicit Self application is unknown")?;
    if application.declaration
        != (s::DeclarationAddress {
            artifact: s::ArtifactRef(0),
            declaration: header.declaration,
        })
        || application.subject != header.self_type
    {
        return Err("source implicit Self input belongs to another actual trait or subject");
    }
    Ok(())
}

fn insert(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    owner: DeclarationRef,
    kind: DefinitionKind,
    header: HeaderRef,
    work: &mut Work,
) -> Result<(), &'static str> {
    declaration(rows, owner, kind, work)?;
    if headers[owner.0 as usize].replace(header).is_some() {
        return Err("source declaration has duplicate or conflicting typed headers");
    }
    Ok(())
}
fn inputs(
    rows: &SourceRows,
    owner: DeclarationRef,
    range: g::Range,
    declared: u32,
    canonical: Option<u32>,
    seen: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(1)?;
    let inputs = range.slice(&rows.clauses)?;
    if declared as usize > inputs.len() {
        return Err("source declared prefix exceeds its complete input range");
    }
    let definition = &rows.declarations[owner.0 as usize];
    for (index, input) in inputs.iter().enumerate() {
        work.charge(1)?;
        let offset = range.start as usize + index;
        if input.owner != owner
            || input.site >= definition.sites
            || std::mem::replace(&mut seen[offset], true)
            || input.self_subject.is_some()
                && canonical.is_none_or(|prefix| index >= prefix as usize)
        {
            return Err("source input lost its owner, site, unique range or canonical Self domain");
        }
    }
    Ok(())
}
fn ordinal(index: usize) -> Result<u32, &'static str> {
    u32::try_from(index).map_err(|_| "source header ordinal overflow")
}
