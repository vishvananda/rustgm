//! Reification checks complete non-type declaration debts in the actual defining scope.
use crate::{
    body::ir::{Body, Rvalue},
    sema::{crates::SourceOriginal, declaration_inputs, model::Declarations},
    support::crates::{MAX_COMPONENTS, generics::source as s},
    types::{Kind, parameters::clauses::Clause},
};
use std::cell::Cell;
pub(super) fn check(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    if view.rows().descriptor().profile() != s::Profile::Source2024ModuleFeaturePredicates {
        return Ok(());
    }
    for value in &body.values {
        *steps = steps.saturating_add(1);
        let Rvalue::Function(definition) = value.kind else {
            continue;
        };
        if !matches!(facts.types.kind(value.ty), Some(Kind::FunctionItem {item,..}) if item.0 == definition.0)
        {
            return Err("source call lost its actual declared function item");
        }
        for input in
            declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), definition)?
        {
            *steps = steps.saturating_add(1);
            let Clause::UnstableFeature(id) = input.value else {
                continue;
            };
            let work = Cell::new(*steps);
            let result = facts.prove_feature(id, Some(view.environment()?), &|count, bytes| {
                work.set(work.get().saturating_add(count));
                if work.get() > MAX_COMPONENTS || bytes != 0 {
                    return Err("source feature call exceeded its prepaid proof envelope");
                }
                Ok(())
            });
            *steps = work.get();
            if !result? {
                return Err("source callable feature requirement lacks its actual defining scope");
            }
        }
    }
    Ok(())
}
