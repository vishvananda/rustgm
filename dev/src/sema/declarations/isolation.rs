//! Census isolation records a failed declaration root and keeps every unit it
//! left in flight unusable. Ordinary checking has no ledger and keeps its first error.
use super::{Checker, DefId, Error, SignaturePhase, State, Work, impl_headers, traits};
use crate::{
    sema::{
        associated::OwnerKind,
        census::{self, Census, Phase},
    },
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
    /// A const body whose typing a failed root left active and suspended.
    Execution(DefId),
}
impl Unit {
    /// The guarded owner a work item enters, whether or not it already began.
    const fn entered(work: Work) -> Option<Self> {
        Some(match work {
            Work::Definition(id, _)
            | Work::AssociatedBounds(id)
            | Work::Signature(_, Some(id), _) => Self::Definition(id),
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
            Work::AssociatedBounds(_) => true,
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
            | Self::CallerLocation(id)
            | Self::Execution(id) => id,
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
    /// The record whose failure left a const body active that this root needs.
    abandoned: Option<usize>,
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
    /// A header whose impl failed, or was blocked, in this census.
    pub(super) fn isolates_header(&self, owner: DefId) -> bool {
        self.poison.contains_key(&Unit::ImplHeader(owner)) || self.settled.contains(&owner)
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
    /// A census-failed impl header never publishes its self type's schema, so
    /// a later use of that self type is blocked by the header's own record.
    pub(super) fn census_isolated_self(&mut self, id: DefId) {
        if let Some(ledger) = self.census.as_deref_mut()
            && ledger.poison.contains_key(&Unit::ImplHeader(id))
        {
            ledger
                .failing
                .get_or_insert(Work::ImplHeader(id, impl_headers::Phase::Begin));
        }
    }
    /// A failed root leaves the const bodies it was typing active, so a later
    /// demand for one is blocked by that root's record, not a dependency cycle.
    pub(super) fn census_abandoned_body(&mut self, id: DefId) {
        if let Some(ledger) = self.census.as_deref_mut()
            && let Some(&cause) = ledger.poison.get(&Unit::Execution(id))
        {
            ledger.abandoned.get_or_insert(cause);
        }
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
            ledger.abandoned = None;
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

    /// The impl that owns the member definition a failing unit entered.
    fn failing_member_owner(&self) -> Result<Option<DefId>, Error> {
        let Some(Unit::Definition(id)) = self
            .census
            .as_deref()
            .and_then(|ledger| ledger.failing)
            .and_then(Unit::entered)
        else {
            return Ok(None);
        };
        let Some(index) = self.facts.associated.owner(id) else {
            return Ok(None);
        };
        let owner = self
            .facts
            .associated
            .source_at(index.0 as usize)
            .map_err(Error::unlocated)?;
        Ok(owner.and_then(|owner| {
            matches!(owner.kind, OwnerKind::Impl { .. }).then_some(owner.self_definition)
        }))
    }

    fn isolate(&mut self, root: DefId, error: Error, exhausted: bool) -> Result<(), Error> {
        // A member of an impl whose header failed is blocked by that header.
        let owner = self.failing_member_owner()?;
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
        let blocker = ledger
            .abandoned
            .take()
            .or_else(|| entered.and_then(|unit| ledger.poison.get(&unit).copied()))
            .or_else(|| {
                owner.and_then(|owner| ledger.poison.get(&Unit::ImplHeader(owner)).copied())
            })
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
        for id in self.execution_bodies.active() {
            ledger.poison.entry(Unit::Execution(id)).or_insert(cause);
        }
        for &work in &self.work {
            if let Some(unit) = Unit::continued(work) {
                ledger.poison.entry(unit).or_insert(cause);
            }
        }
        ledger.trail.clear();
        self.work.clear();
        self.children.clear();
        self.withdraw_unfinished_contracts();
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
