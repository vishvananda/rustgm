//! Owned immutable caller facts accompany a suspended search, without copying its environment.
use super::{Caller, Context, Declarations, Frame, Goal, Query, Search, Selection, continuation};
use crate::{
    resolve::model::DefId,
    sema::traits::selection::{Availability, OwnedCaller},
};

pub(in crate::sema) enum Progress {
    Ready(Selection),
    Pending(DefId, Box<Pending>),
    Inferred { query: Box<Pending> },
}
pub(in crate::sema) struct Pending {
    caller: OwnedCaller,
    query: Query,
}
#[derive(Clone, Copy)]
enum Mode<'a> {
    Plain,
    Normalized,
    Projection(crate::types::projection::Application),
    Refining(Option<&'a [u32]>),
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
    begin_with_inputs(facts, context, goal, caller, availability, Mode::Plain)
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
    begin_with_inputs(facts, context, goal, caller, availability, Mode::Normalized)
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
    )
}

fn begin_with_inputs(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: OwnedCaller,
    availability: Availability,
    mode: Mode<'_>,
) -> Result<Progress, String> {
    if goal.projection_result().is_none()
        && !matches!(mode, Mode::Refining(Some(_)) | Mode::Projection(_))
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
    search.query.refining = matches!(mode, Mode::Refining(_));
    first.inference = if search.query.refining {
        super::frames::Inference::Selecting
    } else {
        super::frames::Inference::Disabled
    };
    if let Mode::Refining(Some(counts)) = mode {
        search.open_universals(facts, &mut first, counts)?;
    }
    let progress = search.begin(facts, first)?;
    match progress {
        continuation::Progress::Ready(decision) => search
            .result(facts, context, goal, decision)
            .map(Progress::Ready),
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
        let Self { caller, query } = self;
        let mut search = Search {
            caller: caller.borrowed(),
            query,
        };
        search.resume_at(first)?;
        search.refresh_inputs(facts, context)?;
        let progress = search.poll(facts)?;
        match progress {
            continuation::Progress::Ready(decision) => search
                .result(facts, context, goal, decision)
                .map(Progress::Ready),
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
