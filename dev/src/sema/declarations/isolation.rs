//! Census isolation records a failed declaration root and keeps every unit it
//! left in flight unusable. Ordinary checking has no ledger and keeps its first error.
use super::{Checker, DefId, Error, SignaturePhase, State, Work, impl_headers, traits};
use crate::{
    sema::census::{self, Census, Phase},
    support::hash::{HashMap, HashSet},
    types::ItemId,
};

/// A guarded declaration state machine keyed by its owner.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Unit {
    Definition(DefId),
    ImplHeader(DefId),
    ValidateImpl(DefId),
    Fields(ItemId),
    Contract(ItemId),
    Environment(DefId),
    SourceContracts(DefId),
    CallerLocation(DefId),
}
impl Unit {
    /// The guarded owner a work item enters, whether or not it already began.
    const fn entered(work: Work) -> Option<Self> {
        Some(match work {
            Work::Definition(id, _) | Work::Signature(_, Some(id), _) => Self::Definition(id),
            Work::ImplHeader(id, _) => Self::ImplHeader(id),
            Work::ValidateImpl(id, _) => Self::ValidateImpl(id),
            Work::Fields(item, _) => Self::Fields(item),
            Work::NominalContract(item, _) => Self::Contract(item),
            Work::ProjectionEnvironment(owner, ..) => Self::Environment(owner),
            Work::SourceResultContracts(id, _) => Self::SourceContracts(id),
            Work::CallerLocation(id, _) => Self::CallerLocation(id),
            _ => return None,
        })
    }
    /// Only a pending continuation proves that its owner began and did not finish.
    /// Not-started siblings left on the stack remain ordinary unseen work.
    const fn continued(work: Work) -> Option<Self> {
        let continued = match work {
            Work::Definition(_, finish)
            | Work::Fields(_, finish)
            | Work::NominalContract(_, finish)
            | Work::ProjectionEnvironment(_, _, finish)
            | Work::CallerLocation(_, finish) => finish,
            Work::Signature(_, Some(_), phase) => !matches!(phase, SignaturePhase::Begin(_)),
            Work::ImplHeader(_, phase) => !matches!(phase, impl_headers::Phase::Begin),
            Work::ValidateImpl(_, phase) => !matches!(
                phase,
                traits::ImplementationPhase::Try | traits::ImplementationPhase::Begin
            ),
            Work::SourceResultContracts(_, phase) => !matches!(phase, traits::MetadataPhase::Begin),
            _ => false,
        };
        if continued { Self::entered(work) } else { None }
    }
    const fn owner(self) -> DefId {
        match self {
            Self::Definition(id)
            | Self::ImplHeader(id)
            | Self::ValidateImpl(id)
            | Self::Environment(id)
            | Self::SourceContracts(id)
            | Self::CallerLocation(id) => id,
            Self::Fields(item) | Self::Contract(item) => DefId(item.0),
        }
    }
}

/// Census-only bookkeeping. It exists only while an analysis-only census runs.
#[derive(Default)]
pub(super) struct Ledger {
    pub(super) census: Census,
    /// Each poisoned unit names the root-cause record that left it in flight.
    poison: HashMap<Unit, usize>,
    /// Roots and failed items that already have a record.
    settled: HashSet<DefId>,
    /// Definitions activated by the current root, in activation order.
    trail: Vec<DefId>,
    /// The innermost work item whose dispatch failed in the current root.
    failing: Option<Work>,
}
impl Ledger {
    pub(super) fn new(census: Census) -> Self {
        Self {
            census,
            ..Self::default()
        }
    }
    pub(super) fn into_census(self) -> Census {
        self.census
    }
}

impl Checker<'_> {
    pub(super) fn census_activate(&mut self, id: DefId) {
        if let Some(ledger) = self.census.as_deref_mut() {
            ledger.trail.push(id);
        }
    }
    pub(super) fn census_failing(&mut self, work: Work) {
        if let Some(ledger) = self.census.as_deref_mut() {
            ledger.failing.get_or_insert(work);
        }
    }
    /// A census-failed definition is never restarted, completed or used.
    pub(super) fn census_blocks(&self, id: DefId) -> bool {
        self.census
            .as_deref()
            .is_some_and(|ledger| ledger.poison.contains_key(&Unit::Definition(id)))
    }
    pub(super) fn census_stops(&self) -> bool {
        self.census
            .as_deref()
            .is_some_and(|ledger| ledger.census.has_failures(Phase::Declarations))
    }
    /// The root-cause record for a propagated error whose failing unit is poisoned.
    pub(super) fn census_blocker(&mut self) -> Option<usize> {
        let ledger = self.census.as_deref_mut()?;
        let unit = ledger.failing.take().and_then(Unit::entered)?;
        ledger.poison.get(&unit).copied()
    }
    /// Every poisoned definition must still be unfinished when the census stops.
    pub(super) fn census_consistent(&self) -> bool {
        self.census.as_deref().is_none_or(|ledger| {
            ledger.poison.keys().all(|unit| match unit {
                Unit::Definition(id) => self.states[id.0 as usize] != State::Complete,
                _ => true,
            })
        })
    }

    /// Drain one root from an empty worklist. In census mode an item failure is
    /// recorded and isolated; ordinary checking returns the same first error.
    pub(super) fn drain_root(
        &mut self,
        root: DefId,
        work: Work,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        if let Some(ledger) = self.census.as_deref_mut() {
            if ledger.settled.contains(&root) {
                return Ok(());
            }
            ledger.failing = None;
        }
        self.work.push(work);
        match self.drain(steps, limit) {
            Ok(()) => {
                if let Some(ledger) = self.census.as_deref_mut() {
                    ledger.trail.clear();
                    ledger.census.checked(Phase::Declarations, 1);
                }
                Ok(())
            }
            Err(error) => self.isolate(root, error, *steps > limit),
        }
    }

    fn isolate(&mut self, root: DefId, error: Error, exhausted: bool) -> Result<(), Error> {
        let Some(mut ledger) = self.census.take() else {
            return Err(error);
        };
        if exhausted || census::exhausts_budget(&error.message) {
            // Budgets are sticky crate state: the census ends here.
            self.census = Some(ledger);
            return Err(error);
        }
        let error = self.report.diagnostic(error);
        let entered = ledger.failing.take().and_then(Unit::entered);
        let blocker = entered
            .and_then(|unit| ledger.poison.get(&unit).copied())
            .or_else(|| ledger.census.repeated(&error));
        let cause = if let Some(cause) = blocker {
            self.settle_blocked(&mut ledger, root, cause);
            cause
        } else {
            let item = entered
                .map(Unit::owner)
                .or_else(|| self.innermost(&ledger.trail))
                .unwrap_or(root);
            let cause = ledger.census.failed(
                Phase::Declarations,
                census::describe(self.report, item),
                &error,
            );
            ledger.settled.insert(item);
            if item != root {
                self.settle_blocked(&mut ledger, root, cause);
            }
            cause
        };
        for &id in &ledger.trail {
            if self.states[id.0 as usize] == State::Active {
                ledger.poison.entry(Unit::Definition(id)).or_insert(cause);
            }
        }
        if let Some(unit) = entered {
            ledger.poison.entry(unit).or_insert(cause);
        }
        for &work in &self.work {
            if let Some(unit) = Unit::continued(work) {
                ledger.poison.entry(unit).or_insert(cause);
            }
        }
        ledger.trail.clear();
        self.work.clear();
        self.children.clear();
        self.census = Some(ledger);
        Ok(())
    }
    fn settle_blocked(&self, ledger: &mut Ledger, root: DefId, cause: usize) {
        if ledger.settled.insert(root) {
            ledger.census.blocked(
                Phase::Declarations,
                census::describe(self.report, root),
                census::site(self.report, root),
                cause,
            );
        }
    }
    /// The newest definition activated by this root that is still unfinished.
    fn innermost(&self, trail: &[DefId]) -> Option<DefId> {
        trail
            .iter()
            .rev()
            .copied()
            .find(|id| self.states[id.0 as usize] == State::Active)
    }
}
