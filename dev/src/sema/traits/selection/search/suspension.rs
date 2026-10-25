//! Owned immutable caller facts accompany a suspended search, without copying its environment.
use super::{Caller, Context, Declarations, Frame, Goal, Query, Search, Selection, continuation};
use crate::sema::traits::selection::{Availability, Dependency, OwnedCaller};

pub(in crate::sema) enum Progress {
    Ready(Selection),
    Pending(Dependency, Box<Pending>),
    Inferred { query: Box<Pending> },
}
pub(in crate::sema) struct Pending {
    caller: OwnedCaller,
    query: Query,
}
/// Optional caller observers of one search.
#[derive(Default)]
struct Hooks<'a> {
    /// Completed concrete proofs shared within one fixed caller and mode.
    reuse: Option<&'a mut super::Reuse>,
    /// Receives whether no caller region identity was allocated.
    untouched: Option<&'a mut bool>,
    /// Cumulative query work, including a failed or still-pending prefix.
    attempted: Option<&'a mut usize>,
}
#[derive(Clone, Copy)]
enum Mode<'a> {
    Plain,
    Normalized,
    Projection(crate::types::projection::Application),
    Refining(Option<&'a [u32]>),
    Canonical(Option<u32>),
}
impl OwnedCaller {
    fn borrowed(&self) -> Caller<'_> {
        Caller {
            assumptions: &self.assumptions,
            environment: self.environment.as_ref(),
            first_region: self.first_region,
            retain_sized: self.retain_sized,
        }
    }
}
pub(in crate::sema) fn begin(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        availability,
        Mode::Plain,
        Hooks::default(),
    )
}

/// Keep a unique partial implementation and all unfinished conditions for body inference.
pub(in crate::sema) fn begin_refining(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
    bound_regions: Option<u32>,
) -> Result<Progress, String> {
    let binders = bound_regions.map(|count| [count]);
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        availability,
        Mode::Refining(binders.as_ref().map(<[u32; 1]>::as_slice)),
        Hooks::default(),
    )
}

/// As [`begin_refining`] without binders, also reporting whether the search
/// left the caller's region allocator untouched (no trial captured a region).
pub(in crate::sema) fn begin_refining_untouched(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
    untouched: &mut bool,
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        availability,
        Mode::Refining(None),
        Hooks {
            reuse: None,
            untouched: Some(untouched),
            attempted: None,
        },
    )
}

/// Preserve every enclosing callable binder while refining one projection parent.
/// Exported input witnesses remain in the caller; universal aliases never escape.
pub(in crate::sema) fn begin_scoped_refining(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
    ambient: &[u32],
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        availability,
        Mode::Refining((!ambient.is_empty()).then_some(ambient)),
        Hooks::default(),
    )
}

/// The enclosing normalizer retains the complete formation proof of these inputs.
pub(in crate::sema) fn begin_normalized(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        availability,
        Mode::Normalized,
        Hooks::default(),
    )
}

/// Parent trait selection and the member's input conditions share one retained proof stack.
pub(in crate::sema) fn begin_projection(
    facts: &mut Declarations,
    context: &mut Context,
    inputs: (&Goal, crate::types::projection::Application),
    caller: OwnedCaller,
    availability: Availability,
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        inputs.0,
        caller,
        availability,
        Mode::Projection(inputs.1),
        Hooks::default(),
    )
}

/// Completed canonical inputs retain the ordinary proof cache across dependency waits.
pub(in crate::sema) fn begin_canonical(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    mode: (Availability, Option<u32>),
    reuse: Option<&mut super::Reuse>,
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        mode.0,
        Mode::Canonical(mode.1),
        Hooks {
            reuse,
            untouched: None,
            attempted: None,
        },
    )
}

pub(in crate::sema) fn begin_executable_counted(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    attempted_query_steps: &mut usize,
) -> Result<Progress, String> {
    begin_with_inputs(
        facts,
        context,
        goal,
        caller,
        Availability::Executable,
        Mode::Canonical(None),
        Hooks {
            attempted: Some(attempted_query_steps),
            ..Hooks::default()
        },
    )
}

pub(super) fn observe_work(observer: &mut Option<&mut usize>, work: usize) {
    if let Some(observer) = observer.as_deref_mut() {
        *observer = (*observer).max(work);
    }
}

fn begin_with_inputs(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
    mode: Mode<'_>,
    hooks: Hooks<'_>,
) -> Result<Progress, String> {
    let Hooks {
        mut reuse,
        untouched,
        mut attempted,
    } = hooks;
    observe_work(&mut attempted, 0);
    if goal.trait_arguments().is_some()
        && !matches!(
            mode,
            Mode::Refining(Some(_)) | Mode::Canonical(Some(_)) | Mode::Projection(_)
        )
    {
        match crate::sema::traits::selection::select_assumption(
            &mut facts.types,
            context,
            goal,
            &caller.borrowed(),
        ) {
            Ok(Selection::Absent) | Err(crate::types::infer::Error::Normalization) => {}
            Ok(proof) => return Ok(Progress::Ready(proof)),
            Err(error) => return Err(error.to_string()),
        }
    }
    let mut first = Frame::new(facts, context, goal, caller.first_region)?;
    if matches!(mode, Mode::Normalized | Mode::Projection(_)) && goal.projection_result().is_none()
    {
        first.normalized_inputs = Some(super::frames::NormalizedInputs { proof: None });
    }
    if let Mode::Projection(application) = mode {
        first.prepare_formation(facts, application)?;
    }
    let mut search = Search::new(caller.borrowed(), availability);
    if let Some(reuse) = reuse.as_deref_mut() {
        if !facts.types.nominal_sizes_sealed() {
            return Err("proof reuse requires sealed nominal sizes".into());
        }
        search.query.reuse = Some(super::completed_proofs::Active::new(std::mem::take(reuse)));
    }
    search.query.refining = matches!(mode, Mode::Refining(_));
    first.inference = if search.query.refining {
        super::frames::Inference::Selecting
    } else {
        super::frames::Inference::Disabled
    };
    if let Mode::Refining(Some(counts)) = mode {
        let result = search.open_universals(facts, &mut first, counts);
        observe_work(&mut attempted, search.query.steps);
        result?;
    }
    if let Mode::Canonical(Some(count)) = mode {
        let result = search.open_universals(facts, &mut first, &[count]);
        observe_work(&mut attempted, search.query.steps);
        result?;
    }
    let progress = search.begin(facts, first);
    observe_work(&mut attempted, search.query.steps);
    let progress = progress?;
    match progress {
        continuation::Progress::Ready(decision) => {
            if let Some(untouched) = untouched {
                *untouched = search.query.allocations.untouched();
            }
            search.return_reuse(facts, decision, reuse)?;
            search
                .result_observed(facts, context, goal, decision, attempted)
                .map(Progress::Ready)
        }
        continuation::Progress::Pending(owner) => {
            let query = search.query;
            Ok(Progress::Pending(
                owner,
                Box::new(Pending { caller, query }),
            ))
        }
        continuation::Progress::Inferred => {
            search.publish_inputs(facts, context)?;
            let query = search.query;
            Ok(Progress::Inferred {
                query: Box::new(Pending { caller, query }),
            })
        }
    }
}
impl Pending {
    /// Reserve every allocated trial ID before declaration work begins.
    pub fn allocation_end(&self) -> u32 {
        self.query.allocations.end()
    }
    pub fn resume(
        self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: &Goal,
        first: u32,
    ) -> Result<Progress, String> {
        self.resume_reusing(facts, context, goal, first, None)
    }
    pub fn resume_counted(
        self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: &Goal,
        first: u32,
        attempted_query_steps: &mut usize,
    ) -> Result<Progress, String> {
        self.resume_observed(
            facts,
            context,
            goal,
            first,
            None,
            Some(attempted_query_steps),
        )
    }
    pub fn resume_reusing(
        self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: &Goal,
        first: u32,
        reuse: Option<&mut super::Reuse>,
    ) -> Result<Progress, String> {
        self.resume_observed(facts, context, goal, first, reuse, None)
    }
    fn resume_observed(
        self,
        facts: &mut Declarations,
        context: &mut Context,
        goal: &Goal,
        first: u32,
        reuse: Option<&mut super::Reuse>,
        mut attempted: Option<&mut usize>,
    ) -> Result<Progress, String> {
        let Self { caller, query } = self;
        let mut search = Search {
            caller: caller.borrowed(),
            query,
        };
        observe_work(&mut attempted, search.query.steps);
        let result = search.resume_at(first);
        observe_work(&mut attempted, search.query.steps);
        result?;
        let result = search.refresh_inputs(facts, context);
        observe_work(&mut attempted, search.query.steps);
        result?;
        let progress = search.poll(facts);
        observe_work(&mut attempted, search.query.steps);
        let progress = progress?;
        match progress {
            continuation::Progress::Ready(decision) => {
                search.return_reuse(facts, decision, reuse)?;
                search
                    .result_observed(facts, context, goal, decision, attempted)
                    .map(Progress::Ready)
            }
            continuation::Progress::Pending(owner) => {
                let query = search.query;
                Ok(Progress::Pending(owner, Box::new(Self { caller, query })))
            }
            continuation::Progress::Inferred => {
                search.publish_inputs(facts, context)?;
                let query = search.query;
                Ok(Progress::Inferred {
                    query: Box::new(Self { caller, query }),
                })
            }
        }
    }
}

impl Search<'_> {
    fn return_reuse(
        &mut self,
        facts: &Declarations,
        decision: super::Decision,
        reuse: Option<&mut super::Reuse>,
    ) -> Result<(), String> {
        if let Some(reuse) = reuse {
            self.query.export_completed(facts, decision)?;
            if let Some(active) = self.query.reuse.take() {
                *reuse = active.reuse;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/waited_queries.rs"]
mod tests;
