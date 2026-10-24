//! Inspect the unchanged complete provider, not a reduced or fabricated source body.
use super::*;
use crate::support::crates::{generics::source as s, owner_catalog::DeclarationRef};

#[test]
fn complete_checked_provider_retains_required_and_provided_members_and_inputs() -> Result {
    println!(
        "SourceTraitHeader={} SourceClause={} SourceCatalog={} bindings-imported-role={:?}",
        size_of::<s::TraitHeader>(),
        size_of::<s::Clause>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        crate::sema::compiler_items::Catalog::binding_layouts()
    );
    let (report, analysis, identity) = analyzed_provider()?;
    let mut facts = analysis.facts;
    let observed = inspect_source_transport(&report, &mut facts, &identity, analysis.prepared)?;
    let rows = &observed.rows;
    supertrait_inputs(rows)?;
    assert_eq!(rows.traits.len(), 8);
    assert_eq!(rows.functions.len(), 4);
    assert_eq!(rows.implementations.len(), 4);
    assert_eq!(rows.roles.len(), 9);
    assert_eq!(rows.originals.len(), 3);
    assert_eq!(rows.body_owners.len(), 3);
    assert_eq!(rows.body_inputs.len(), 3);
    assert_eq!(
        rows.functions
            .iter()
            .filter(|function| function.body == s::FunctionBody::Absent)
            .count(),
        1
    );
    for (index, original) in rows.originals.iter().enumerate() {
        assert_eq!(original.body.0 as usize, index);
        assert_eq!(rows.body_inputs[index].body, original.body);
    }
    let (from, original) = provided(rows, "clone_from")?;
    assert_eq!(
        from.constness,
        crate::support::constness::Constness::Ordinary
    );
    assert_eq!(original.context, s::CallContext::ConstDefinition);
    assert_eq!(original.projections.len(), 2);
    assert_eq!(original.calls.len(), 1);
    assert_eq!(
        original.calls[0].effect,
        crate::support::constness::Constness::Conditional
    );
    let clone = role(rows, LangItem::Clone)?;
    let destruct = role(rows, LangItem::Destruct)?;
    let inputs = &rows.body_inputs[original.body.0 as usize];
    assert!(
        matches!(inputs.environment, s::EnvironmentOrigin::ConstTraitDefault {member, trait_}
        if member == from.declaration && trait_.declaration == clone && trait_.artifact == s::ArtifactRef(0))
    );
    assert!(original.variables.iter().any(|variable| matches!(
        variable.allocation,
        s::RegionAllocation::Inference {
            needs_provenance: true
        }
    )));
    assert!(from.inputs.slice(&rows.clauses)?.iter().any(|clause| matches!(clause.requirement,
        s::Requirement::Trait {application, effect: crate::support::constness::Constness::Conditional}
        if rows.trait_applications[application.0 as usize].declaration.declaration == destruct)));
    ordinary_inputs(rows)?;
    identity_inputs(rows)?;
    println!(
        "source-producer {:?} source-original={} owner={} owner-kind={} node-key-option={} predicate-option={} value={}",
        observed.work,
        size_of::<s::Original>(),
        size_of::<crate::sema::associated::Owner>(),
        size_of::<crate::sema::associated::OwnerKind>(),
        size_of::<Option<crate::resolve::unit::NodeKey>>(),
        size_of::<Option<crate::types::parameters::predicates::Predicate>>(),
        size_of::<body::ir::Value>()
    );
    println!(
        "SourceCatalog={} bindings-imported-role={:?}",
        size_of::<crate::sema::compiler_items::Catalog>(),
        crate::sema::compiler_items::Catalog::binding_layouts()
    );
    assert_eq!(size_of::<crate::sema::associated::Owner>(), 32);
    assert_eq!(size_of::<crate::sema::associated::OwnerKind>(), 20);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert!(
        observed
            .work
            .reserved_bytes
            .saturating_add(observed.work.scratch_bytes)
            <= crate::support::crates::MAX_BYTES
    );
    Ok(())
}

fn ordinary_inputs(rows: &s::SourceRows) -> Result {
    let (_, original) = provided(rows, "clone")?;
    assert_eq!(original.context, s::CallContext::Runtime);
    let inputs = &rows.body_inputs[original.body.0 as usize];
    assert_eq!(inputs.environment, s::EnvironmentOrigin::Contract);
    let builtins = inputs
        .builtins
        .as_ref()
        .ok_or("missing ordinary source Copy inputs")?;
    assert_eq!(builtins.environment, s::EnvironmentOrigin::Contract);
    assert!(
        inputs
            .normalizations
            .iter()
            .any(|input| !input.owned.is_empty() && !input.equalities.is_empty())
    );
    assert_eq!(
        builtins
            .source_copy
            .as_ref()
            .ok_or("missing ordinary Copy queries")?
            .goals
            .len(),
        1
    );
    Ok(())
}
fn identity_inputs(rows: &s::SourceRows) -> Result {
    let (function, original) = provided(rows, "identity")?;
    let inputs = &rows.body_inputs[original.body.0 as usize];
    let queries = inputs
        .builtins
        .as_ref()
        .and_then(|builtins| builtins.source_copy.as_ref())
        .ok_or("missing identity move/Copy query")?;
    assert_eq!(
        queries
            .declaration
            .map(|declaration| declaration.declaration),
        Some(role(rows, LangItem::Copy)?)
    );
    assert_eq!(queries.goals.len(), 1);
    for clause in function.inputs.slice(&rows.clauses)? {
        if let s::Requirement::Trait { application, .. } = clause.requirement {
            assert_ne!(
                rows.trait_applications[application.0 as usize]
                    .declaration
                    .declaration,
                role(rows, LangItem::Copy)?
            );
        }
    }
    Ok(())
}
fn role(rows: &s::SourceRows, role: LangItem) -> Result<DeclarationRef> {
    rows.roles
        .iter()
        .find(|binding| binding.role == role)
        .map(|binding| binding.declaration.declaration)
        .ok_or_else(|| "missing actual role".into())
}
fn provided<'a>(
    rows: &'a s::SourceRows,
    name: &str,
) -> Result<(&'a s::FunctionHeader, &'a s::Original)> {
    let function = rows
        .functions
        .iter()
        .find(|function| {
            matches!(function.body, s::FunctionBody::Original(_))
                && rows.declarations[function.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some(name)
        })
        .ok_or("missing actual provided function")?;
    let s::FunctionBody::Original(body) = function.body else {
        return Err("missing provided original identity".into());
    };
    Ok((function, &rows.originals[body.0 as usize]))
}

fn supertrait_inputs(rows: &s::SourceRows) -> Result {
    let meta = role(rows, LangItem::MetaSized)?;
    for (kind, literal, count) in [
        (LangItem::Clone, 1, 2),
        (LangItem::Freeze, 0, 1),
        (LangItem::Copy, 1, 2),
    ] {
        let declaration = role(rows, kind)?;
        let header = rows
            .traits
            .iter()
            .find(|header| header.declaration == declaration)
            .ok_or("source trait row")?;
        let inputs = header.inputs.slice(&rows.clauses)?;
        assert_eq!(header.canonical_inputs, count);
        assert!(header.canonical_inputs < header.declared_inputs);
        assert!(header.declared_inputs as usize <= inputs.len());
        assert_eq!(
            header
                .written_supertraits
                .slice(&rows.clause_indices)?
                .len(),
            literal
        );
        assert!(
            inputs[..count as usize]
                .iter()
                .all(|clause| clause.self_subject.is_some())
        );
        assert!(
            inputs[count as usize..]
                .iter()
                .all(|clause| clause.self_subject.is_none())
        );
        assert!(inputs[..count as usize].iter().any(|clause| matches!(clause.requirement,
            s::Requirement::Trait { application, .. } if rows.trait_applications[application.0 as usize].declaration.declaration == meta)));
        let s::Requirement::Trait { application, .. } = inputs[count as usize].requirement else {
            return Err("implicit own-trait input lost its kind".into());
        };
        assert_eq!(
            rows.trait_applications[application.0 as usize]
                .declaration
                .declaration,
            declaration
        );
    }
    Ok(())
}
