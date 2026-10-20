//! Caller-visible opaque promises are mapped without reading hidden representations.
use super::{Assumptions, Declarations};
use crate::types::{GenericArgs, ItemId, TypeId, opaque::OpaqueId, outlives::ResultId};
use std::sync::Arc;

pub(super) fn traits(
    facts: &mut Declarations,
    ty: TypeId,
    id: OpaqueId,
    arguments: GenericArgs,
    item: ItemId,
) -> Result<(Arc<Assumptions>, usize), String> {
    let (templates, template_steps) =
        super::templates::for_declaration(facts, ResultId::Opaque(id))?;
    let mut result = Assumptions::default();
    let mut cost = 0;
    if let Some(predicates) = templates.candidates(item) {
        let declaration = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing opaque result contract")?;
        let mut fold = declaration.substitution(&facts.types, arguments)?;
        for &clause in predicates {
            let mapped = fold.clause(&mut facts.types, clause)?;
            result
                .insert_clause(&facts.types, mapped)
                .map_err(|e| e.to_string())?;
        }
        cost = fold.steps.saturating_add(predicates.len());
    }
    let result = Arc::new(result);
    facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing opaque rule contracts")?
        .result_rules
        .insert(ty, item, Arc::clone(&result), cost)?;
    Ok((result, cost.saturating_add(template_steps)))
}

pub(in crate::sema) fn prepare_outlives(facts: &mut Declarations) -> Result<(), String> {
    let mut ids: Vec<_> = facts.opaques.declarations.keys().copied().collect();
    ids.sort_unstable_by_key(|id| id.0);
    for id in ids {
        let (templates, _) = super::templates::for_declaration(facts, ResultId::Opaque(id))?;
        let application = facts.opaques.declarations[&id].application;
        facts
            .types
            .declare_opaque_outlives(application, &templates.outlives)?;
    }
    Ok(())
}
