//! Restore structural parent arguments without turning coherence equalities into proof.
use super::{
    proof_inputs::Inputs,
    retention_work::{Budget, Counted},
};
use crate::{
    sema::model::Declarations,
    support::hash::HashMap,
    types::{
        GenericArgs, RegionId,
        arguments::Argument,
        lifetime_proof::Obligations,
        substitute::{self, RegionEquations},
    },
};
pub(super) fn close(
    facts: &mut Declarations,
    inputs: &Inputs,
    proof: &Obligations,
    arguments: GenericArgs,
    budget: &Budget,
) -> Result<GenericArgs, String> {
    let count = facts
        .types
        .arguments(arguments)
        .ok_or("specialization parent arguments disappeared")?
        .len();
    let equation_count = inputs
        .context
        .region_equalities
        .len()
        .saturating_add(proof.equalities.len());
    budget.charge(
        1 + count + equation_count,
        4 * count * size_of::<Argument>()
            + crate::support::retained::new_map::<RegionId, RegionId>(
                inputs
                    .child_regions
                    .len()
                    .saturating_add(inputs.parent_regions.len())
                    .saturating_add(2 * equation_count),
            ),
    )?;
    let mut equations = RegionEquations::default();
    let mut work = 0;
    let result = equations.extend_counted(
        inputs
            .context
            .region_equalities
            .iter()
            .copied()
            .chain(proof.root_equalities()),
        &mut work,
    );
    budget.charge(work, 0)?;
    result?;
    let mut replacements = HashMap::default();
    for (index, &region) in inputs.child_regions.iter().enumerate() {
        replacements.insert(
            region,
            RegionId::bound(
                0,
                u16::try_from(index)
                    .map_err(|_| "specialization child capture exceeds binder range")?,
            )?,
        );
    }
    let mut values: Vec<_> = facts
        .types
        .arguments(arguments)
        .ok_or("specialization parent arguments disappeared")?
        .collect();
    let mut observer = Counted::new(
        substitute::Bound::observe_budgeted_regions(1_000_000, 64 * 1024 * 1024)?,
        budget,
    )?;
    for &argument in &values {
        observer.apply_argument(&mut facts.types, argument)?;
    }
    observer.pay()?;
    for region in observer
        .observed_regions()
        .chain(inputs.parent_regions.iter().copied())
    {
        if region == RegionId::STATIC
            || region.bound_parts().is_some()
            || replacements.contains_key(&region)
        {
            continue;
        }
        let mut attempted = 0;
        let result = equations.canonical_anchor(region, &mut attempted);
        budget.charge(attempted, 0)?;
        let anchor = result?;
        let canonical = if anchor == RegionId::STATIC || anchor.bound_parts().is_some() {
            anchor
        } else {
            *replacements
                .get(&anchor)
                .ok_or("specialization parent capture has no unique child anchor")?
        };
        budget.charge(
            1,
            crate::support::retained::new_map::<RegionId, RegionId>(1),
        )?;
        replacements.insert(region, canonical);
    }
    let mut fold = Counted::new(substitute::Bound::restore_regions(replacements), budget)?;
    for argument in &mut values {
        *argument = fold.apply_argument(&mut facts.types, *argument)?;
    }
    fold.pay()?;
    require_closed(facts, &values, budget)?;
    facts.types.intern_arguments(&values).map_err(Into::into)
}

fn require_closed(
    facts: &mut Declarations,
    values: &[Argument],
    budget: &Budget,
) -> Result<(), String> {
    // Inspect the completed actual mapping; unanchored private witnesses never
    // escape as permanent graph arguments. Static and nested binders remain real.
    let mut check = Counted::new(
        substitute::Bound::observe_budgeted_regions(1_000_000, 64 * 1024 * 1024)?,
        budget,
    )?;
    for &argument in values {
        check.apply_argument(&mut facts.types, argument)?;
    }
    check.pay()?;
    if check
        .observed_regions()
        .any(|region| region != RegionId::STATIC && region.bound_parts().is_none())
    {
        return Err("specialization mapping retains a private query region".into());
    }
    Ok(())
}
