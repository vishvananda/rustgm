//! Owned immutable caller facts accompany a suspended search, without copying its environment.
use super::{Caller, Context, Declarations, Frame, Goal, Query, Search, Selection, continuation};
use crate::{
    resolve::model::DefId,
    sema::traits::selection::{Availability, OwnedCaller},
};

pub(in crate::sema) enum Progress {
    Ready(Selection),
    Pending(DefId, Box<Pending>),
}
pub(in crate::sema) struct Pending {
    caller: OwnedCaller,
    query: Query,
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
    if goal.projection_result().is_none() {
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
    let first = Frame::new(facts, context, goal, caller.first_region)?;
    let mut search = Search::new(caller.borrowed(), availability);
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
        let progress = search.poll(facts)?;
        match progress {
            continuation::Progress::Ready(decision) => search
                .result(facts, context, goal, decision)
                .map(Progress::Ready),
            continuation::Progress::Pending(owner) => {
                let query = search.query;
                Ok(Progress::Pending(owner, Box::new(Self { caller, query })))
            }
        }
    }
}
