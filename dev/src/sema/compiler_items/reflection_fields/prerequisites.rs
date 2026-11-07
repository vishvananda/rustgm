//! Field eligibility and actual authored supertraits become ordinary scoped children.
use super::{Declarations, Description, LangItem};
use crate::types::{
    GenericArgs, ItemId,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    traits::Application,
};

pub(in crate::sema) fn requirements(
    facts: &mut Declarations,
    goal: Predicate,
    description: Description,
) -> Result<Vec<Predicate>, String> {
    let parent = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::Field))
        .ok_or("Field lost its parent declaration")?;
    let header = super::authentication::canonical_header(facts, parent)?;
    let self_id = crate::types::parameters::argument_id(
        &facts.types,
        crate::types::arguments::Argument::Type(header.self_type),
    )?;
    let inputs = header.inputs.view();
    let count = inputs.len();
    let capacity = count.saturating_add(2);
    let bytes = count
        .saturating_mul(size_of::<Clause>())
        .saturating_add(capacity.saturating_mul(size_of::<Predicate>()));
    crate::sema::header_attributes::charge_source_retention(facts, capacity, bytes)
        .map_err(|e| e.to_string())?;
    let clauses = inputs.iter().map(|input| input.value).collect::<Vec<_>>();
    let mut result = Vec::with_capacity(capacity);
    let pay = crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|e| e.to_string())?;
    let mut fold = crate::types::substitute::Bound::parameters_captured_charged(
        &facts.types,
        &[(self_id, crate::types::arguments::Argument::Type(goal.ty))],
        Vec::new(),
        &|work, bytes| pay.charge(work, bytes),
    )?;
    for clause in clauses {
        let Clause::Predicate(predicate) =
            fold.clause_charged(&mut facts.types, clause, &|work, bytes| {
                pay.charge(work, bytes)
            })?
        else {
            return Err(
                "Field quantified or feature input proof requires its full scoped frontier".into(),
            );
        };
        result.push(predicate);
    }
    let sized = facts
        .source_bindings
        .as_deref()
        .ok_or("Field sizing lost its source catalog")?
        .header_role(LangItem::Sized)?
        .ok_or("Field requires the actual source Sized trait")?;
    let application = Application {
        item: ItemId(sized.definition.0),
        arguments: GenericArgs::EMPTY,
    };
    let application = if let Some(existing) = facts.types.find_trait_application(application) {
        existing
    } else {
        let growth = facts.types.trait_application_growth();
        pay.charge(growth.work.saturating_add(1), growth.bytes)?;
        facts.types.intern_trait_application(application)?
    };
    for ty in [description.base, description.field] {
        result.push(Predicate {
            ty,
            bound: Bound::Trait(application),
        });
    }
    crate::sema::header_attributes::charge_source_contract_query(facts, fold.steps)
        .map_err(|e| e.to_string())?;
    Ok(result)
}
