//! Selected implementation templates share inputs while their premises remain pending.
use super::{Caller, Context, Declarations, inference, instantiation};
use crate::{
    resolve::model::DefId,
    sema::{
        projections::definition,
        traits::{heads::Subject, implementations::Availability},
    },
    types::{
        infer::{Error, TermId, arguments::Argument, probe::Probe},
        lifetime_proof::Obligations,
        substitute::Bound,
    },
};

struct Candidate {
    probe: Probe,
    instance: instantiation::Instance,
    arguments: Vec<Argument>,
    proof: Obligations,
}

/// Header compatibility must be unique before any associated result is inspected.
/// A compatible header which refines inputs still counts against uniqueness.
/// This returns structural equations only; the body retains the original query
/// until the ordinary selector proves every premise with fully determined inputs.
pub(in crate::sema) fn symbolic_result(
    facts: &mut Declarations,
    source: &mut Context,
    root: TermId,
    caller: &Caller<'_>,
) -> Result<Option<(TermId, Obligations)>, String> {
    let application = source
        .projection_application(&facts.types, root)
        .map_err(inference)?;
    let goal = &application.predicate;
    let (item, _) = goal
        .trait_arguments()
        .ok_or("missing projected trait goal")?;
    let Some(subject) = Subject::query(&facts.types, source, goal.subject()).map_err(inference)?
    else {
        return Ok(None);
    };
    if caller.assumptions.may_have_subject(item, subject) {
        return Ok(None);
    }
    let Some(registry) = &mut facts.generics.implementations else {
        return Ok(None);
    };
    let crate::sema::traits::implementations::Candidates::Ready(owners) = registry
        .candidates(&mut facts.types, source, goal, Availability::Validated)
        .map_err(inference)?
    else {
        return Ok(None);
    };
    if let [owner] = owners.as_slice()
        && let Some(inputs) =
            super::header_inputs::independent(facts, source, goal, *owner).map_err(inference)?
    {
        let definition = definition(facts, *owner, application.item)?;
        let result = facts.infer_associated_definition(
            source,
            definition,
            &application.arguments,
            vec![(inputs.parameter, Argument::Type(inputs.value))],
            inputs.regions,
        )?;
        return Ok(Some((result, Obligations::new(caller.first_region))));
    }
    let mut selected = None;
    for owner in owners {
        if let Some(candidate) = Candidate::new(facts, source, root, owner, caller.first_region)? {
            if selected.is_some() {
                return Ok(None);
            }
            selected = Some(candidate);
        }
    }
    let Some(mut selected) = selected else {
        return Ok(None);
    };
    let before = selected.probe.context().steps;
    let result = selected.export(facts, source, application.item);
    source
        .account_work(selected.probe.context().steps.saturating_sub(before))
        .map_err(inference)?;
    Ok(result?.map(|term| (term, selected.proof)))
}

impl Candidate {
    fn new(
        facts: &mut Declarations,
        source: &mut Context,
        root: TermId,
        owner: DefId,
        first: u32,
    ) -> Result<Option<Self>, String> {
        let (mut probe, roots) = Probe::new(&facts.types, source, &[root]).map_err(inference)?;
        let mut fold = Bound::open_regions(first)?;
        let root = probe
            .context()
            .map_regions(&mut facts.types, roots[0], &mut fold)
            .map_err(inference)?;
        let mut proof = Obligations::new(first);
        proof.regions.end = fold
            .next_region()
            .ok_or("missing symbolic projection region interval")?;
        proof.equalities.extend(
            fold.opened_regions()
                .map(|(original, free)| (free, original)),
        );
        proof
            .equalities
            .sort_unstable_by_key(|&(free, original)| (free.0, original.0));
        let application = probe
            .context()
            .projection_application(&facts.types, root)
            .map_err(inference)?;
        let instance =
            instantiation::instantiate(facts, probe.context(), owner, proof.regions.end)?;
        proof.regions.end +=
            u32::try_from(instance.regions.len()).map_err(|_| "impl region count overflow")?;
        let compatible =
            probe
                .context()
                .equate_predicates(&facts.types, &application.predicate, &instance.goal);
        source
            .account_work(probe.context().steps)
            .map_err(inference)?;
        match compatible {
            Ok(()) => Ok(Some(Self {
                probe,
                instance,
                arguments: application.arguments,
                proof,
            })),
            Err(Error::Mismatch | Error::Occurs) => Ok(None),
            Err(error) => Err(inference(error)),
        }
    }

    fn export(
        &mut self,
        facts: &mut Declarations,
        source: &mut Context,
        item: crate::types::ItemId,
    ) -> Result<Option<TermId>, String> {
        if !self.probe.preserves_inputs(source).map_err(inference)? {
            return Ok(None);
        }
        let definition = definition(facts, self.instance.owner, item)?;
        let result = facts.infer_associated_definition(
            self.probe.context(),
            definition,
            &self.arguments,
            std::mem::take(&mut self.instance.bindings),
            std::mem::take(&mut self.instance.regions),
        )?;
        self.proof
            .equalities
            .append(&mut self.probe.context().region_equalities);
        let mut restoration = self.proof.restoration()?;
        let result = self
            .probe
            .context()
            .map_regions(&mut facts.types, result, &mut restoration)
            .map_err(inference)?;
        self.probe
            .export(&facts.types, source, result)
            .map_err(inference)
    }
}
