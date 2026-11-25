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

type Ledger = (Vec<Option<HeaderRef>>, s::bodymap::Shared);
pub(super) fn validate(rows: &SourceRows, work: &mut Work) -> Result<Ledger, &'static str> {
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
    nominal_inputs(rows, &mut headers, &mut clauses, work)?;
    associated_headers(rows, &mut headers, &mut clauses, work)?;
    function_headers(rows, &mut headers, &mut clauses, work)?;
    opaque_declarations(rows, &headers, &mut clauses, work)?;
    super::super::free_constants::headers(rows, &mut headers, work)?;
    alias_headers(rows, &mut headers, work)?;
    trait_alias_headers(rows, &mut headers, &mut clauses, work)?;
    static_headers(rows, &mut headers, work)?;
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        if matches!(
            row.definition.kind,
            DefinitionKind::Function
                | DefinitionKind::Trait
                | DefinitionKind::Struct
                | DefinitionKind::Enum
                | DefinitionKind::Variant
                | DefinitionKind::AssociatedType
                | DefinitionKind::Constant
                | DefinitionKind::TypeAlias
                | DefinitionKind::TraitAlias
                | DefinitionKind::Static
        ) && headers[index].is_none()
        {
            return Err("source declaration has no complete typed header");
        }
        if row.definition.kind == DefinitionKind::SelfType && headers[index].is_none() {
            let parent = row
                .definition
                .parent
                .and_then(|parent| headers.get(parent.0 as usize));
            let expected = match parent {
                Some(Some(HeaderRef::Trait(trait_))) => {
                    rows.traits[*trait_ as usize].self_definition
                }
                Some(Some(HeaderRef::TraitAlias(alias))) => {
                    rows.trait_aliases()[*alias as usize].self_definition
                }
                _ => return Err("source Self declaration has no actual lexical owner"),
            };
            if expected.0 as usize != index {
                return Err("source trait or alias has an extra foreign Self declaration");
            }
        }
    }
    work.charge(clauses.len())?;
    if clauses.iter().any(|used| !used) {
        return Err("source clauses have incomplete owning header coverage");
    }
    member_rows::validate(rows, &headers, work)?;
    let bodies = s::bodymap::Lookup::build(rows, work)?;
    body_coverage::validate(rows, &headers, bodies.as_deref(), work)?;
    formal_rows::validate(rows, &headers, bodies.as_deref(), work)?;
    Ok((headers, bodies))
}

fn alias_headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut previous = None;
    for (index, header) in rows.type_aliases().iter().enumerate() {
        insert(
            rows,
            headers,
            header.declaration,
            DefinitionKind::TypeAlias,
            HeaderRef::TypeAlias(ordinal(index)?),
            work,
        )?;
        let row = &rows.declarations[header.declaration.0 as usize];
        let parent = row
            .definition
            .parent
            .and_then(|id| rows.declarations.get(id.0 as usize));
        if parent.is_none_or(|parent| parent.definition.kind != DefinitionKind::Module)
            || previous.is_some_and(|ordinal| ordinal >= header.declaration.0)
        {
            return Err("source type alias lacks canonical order or its actual module declaration");
        }
        previous = Some(header.declaration.0);
    }
    Ok(())
}

fn trait_alias_headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    clauses: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut previous = None;
    for (index, header) in rows.trait_aliases().iter().enumerate() {
        insert(
            rows,
            headers,
            header.declaration,
            DefinitionKind::TraitAlias,
            HeaderRef::TraitAlias(ordinal(index)?),
            work,
        )?;
        declaration(rows, header.self_definition, DefinitionKind::SelfType, work)?;
        let parent = rows.declarations[header.declaration.0 as usize]
            .definition
            .parent
            .and_then(|id| rows.declarations.get(id.0 as usize));
        if parent.is_none_or(|parent| parent.definition.kind != DefinitionKind::Module)
            || rows.declarations[header.self_definition.0 as usize]
                .definition
                .parent
                != Some(header.declaration)
            || previous.is_some_and(|ordinal| ordinal >= header.declaration.0)
        {
            return Err("source trait alias changed its actual module or lexical Self owner");
        }
        previous = Some(header.declaration.0);
        let declared = u32::try_from(header.inputs.slice(&rows.clauses)?.len())
            .map_err(|_| "source alias input count overflow")?;
        inputs(
            rows,
            header.declaration,
            header.inputs,
            declared,
            None,
            clauses,
            work,
        )?;
        for input in header.inputs.slice(&rows.clauses)? {
            work.charge(1)?;
            if input.binder.is_some()
                || !matches!(
                    input.requirement,
                    s::Requirement::Trait {
                        effect: crate::support::constness::Constness::Ordinary,
                        ..
                    } | s::Requirement::ProjectionEqual { .. }
                )
            {
                return Err("source trait alias requires its direct ordinary input profile");
            }
        }
    }
    Ok(())
}

fn static_headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(statics) = rows.cold().statics.as_deref() else {
        return Ok(());
    };
    let mut previous = None;
    for (index, header) in statics.headers.iter().enumerate() {
        insert(
            rows,
            headers,
            header.declaration,
            DefinitionKind::Static,
            HeaderRef::Static(ordinal(index)?),
            work,
        )?;
        let row = &rows.declarations[header.declaration.0 as usize];
        let parent = row
            .definition
            .parent
            .and_then(|id| rows.declarations.get(id.0 as usize));
        if parent.is_none_or(|owner| {
            !matches!(
                owner.definition.kind,
                DefinitionKind::Module | DefinitionKind::Function
            )
        }) || previous.is_some_and(|ordinal| ordinal >= header.declaration.0)
        {
            return Err(
                "source static lacks canonical order or its actual module or function owner",
            );
        }
        previous = Some(header.declaration.0);
    }
    Ok(())
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
    } = &implicit.requirement
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
        if matches!(input.requirement, s::Requirement::UnstableFeature(_))
            && (input.binder.is_some()
                || input.self_subject.is_some()
                || index >= declared as usize
                || canonical.is_some_and(|count| index >= count as usize))
        {
            return Err("feature predicate is not a genuine unquantified declared input");
        }
    }
    Ok(())
}
fn ordinal(index: usize) -> Result<u32, &'static str> {
    u32::try_from(index).map_err(|_| "source header ordinal overflow")
}

fn nominal_inputs(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    clauses: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    super::super::nominal_contracts::headers(rows, headers, work)?;
    super::super::sum_contracts::headers(rows, headers, work)?;
    for header in &rows.nominals {
        inputs(
            rows,
            header.declaration,
            header.inputs,
            header.declared_inputs,
            None,
            clauses,
            work,
        )?;
    }
    Ok(())
}

fn associated_headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    clauses: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    for (index, header) in rows.associated_types.iter().enumerate() {
        insert(
            rows,
            headers,
            header.declaration,
            DefinitionKind::AssociatedType,
            HeaderRef::AssociatedType(ordinal(index)?),
            work,
        )?;
        if header.scope != header.declaration
            || rows.declarations[header.declaration.0 as usize]
                .definition
                .parent
                != Some(header.parent)
            || !matches!(
                headers.get(header.parent.0 as usize),
                Some(Some(HeaderRef::Trait(_)))
            )
        {
            return Err("associated type header lost its actual trait/scope ownership");
        }
        inputs(
            rows,
            header.declaration,
            header.inputs,
            header.declared_inputs,
            None,
            clauses,
            work,
        )?;
        inputs(
            rows,
            header.declaration,
            header.bounds,
            header.bounds.end.saturating_sub(header.bounds.start),
            None,
            clauses,
            work,
        )?;
    }
    for (index, row) in rows.associated_definitions().iter().enumerate() {
        insert(
            rows,
            headers,
            row.declaration,
            DefinitionKind::AssociatedType,
            HeaderRef::AssociatedDefinition(ordinal(index)?),
            work,
        )?;
        let parent = rows.declarations[row.declaration.0 as usize]
            .definition
            .parent;
        if !parent.is_some_and(|parent| {
            matches!(
                headers.get(parent.0 as usize),
                Some(Some(HeaderRef::Implementation(_)))
            )
        }) || row.value.0 as usize >= rows.types.len()
        {
            return Err("associated result definition lost its actual implementation owner");
        }
    }
    let mut previous = None;
    for row in &rows.cold().associated_definition_inputs {
        work.charge(1)?;
        if previous.is_some_and(|index| index >= row.index) || row.inputs.start >= row.inputs.end {
            return Err("associated definition inputs are empty or out of declaration order");
        }
        previous = Some(row.index);
        let declaration = rows
            .associated_definitions()
            .get(row.index as usize)
            .ok_or("associated definition inputs name an unknown result")?
            .declaration;
        inputs(
            rows,
            declaration,
            row.inputs,
            row.inputs.end - row.inputs.start,
            None,
            clauses,
            work,
        )?;
    }
    Ok(())
}

fn function_headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    clauses: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    for (index, header) in rows.functions.iter().enumerate() {
        insert(
            rows,
            headers,
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
            clauses,
            work,
        )?;
    }
    Ok(())
}

fn opaque_declarations(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    clauses: &mut [bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut previous = None;
    for decl in &rows.cold().opaques.declarations {
        work.charge(1)?;
        let key = (decl.declaration.0, decl.ordinal);
        if previous.is_some_and(|prior| prior >= key)
            || !matches!(
                headers.get(decl.declaration.0 as usize),
                Some(Some(HeaderRef::Function(_)))
            )
        {
            return Err("source opaque declaration lacks canonical order or a function owner");
        }
        previous = Some(key);
        inputs(
            rows,
            decl.declaration,
            decl.bounds,
            decl.bounds.end.saturating_sub(decl.bounds.start),
            None,
            clauses,
            work,
        )?;
    }
    Ok(())
}
