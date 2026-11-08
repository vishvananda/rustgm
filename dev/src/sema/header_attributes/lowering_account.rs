//! Lowering pays its own input-proportional account on the crate's ledger.
//!
//! Header analysis and lowering share one ledger, but not one bound: lowering
//! retains fixed proof and row slots per original body as well as work per
//! input element, so its account grants a per-element term and a per-body-owner
//! term, measured in `docs/parallel/lib193-header-budget.md`. Only work and
//! storage the originals size pay here. Demanded bodies pay the fixed demand
//! budget by their own output: while one is traversed or emitted, a lexical
//! demand scope collects every charge that would reach this account. Neither
//! constant may be raised to absorb repeated or superlinear work.
use super::{Declarations, InputBound, Payments, Report, input_graph, retained};
use crate::{
    resolve::model::DefKind,
    syntax::ast::{BlockFlavor, Kind, TypeFlavor},
};
use std::cell::Cell;

/// Lowering work units per input-graph element. With each body owner paid
/// separately, the measured residual is at most 0.09 per element.
const WORK_PER_INPUT: usize = 4;
/// Lowering retained bytes per input-graph element: a floor for payers that
/// scale with body size. With each body owner paid separately, the measured
/// residual is zero.
const BYTES_PER_INPUT: usize = 32;
/// Lowering work units per input body owner: more than twice the worst
/// measured 27, a trivial `const fn` whose lowering also asks 17 header queries.
const WORK_PER_BODY_OWNER: usize = 64;
/// Lowering retained bytes per input body owner: twice the worst measured
/// 768, one body's 480-byte proof slot in the exact proof reservation plus
/// `Originals::take`'s ceiling of three 96-byte row slots (the moved row
/// vector after doubling and its exact replacement).
const BYTES_PER_BODY_OWNER: usize = 1536;

pub(super) struct Account {
    steps: Cell<usize>,
    bytes: Cell<usize>,
    bound: InputBound,
    /// The work and bytes charged inside the open demand scope, which its
    /// opener pays to the demand budget when the scope closes.
    scope: Cell<Option<[usize; 2]>>,
    /// The first demanded proof index and the bytes the demand traversal left
    /// on its budget, carried from the traversal to emission.
    carried: Cell<Option<[usize; 2]>>,
}
impl Account {
    pub(super) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        if let Some([scoped_steps, scoped_bytes]) = self.scope.get() {
            self.scope.set(Some([
                scoped_steps.saturating_add(steps),
                scoped_bytes.saturating_add(bytes),
            ]));
            return Ok(());
        }
        self.steps.set(self.steps.get().saturating_add(steps));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.bound.exceeded(self.steps.get(), self.bytes.get()) {
            return Err("lowering work or retention exceeds its input-graph bound");
        }
        Ok(())
    }
}

/// Open lowering's account on the crate's existing ledger, once. The report
/// must be the one whose input graph the declarations snapshotted; a crate
/// without a ledger keeps paying nothing new here.
pub(super) fn open(report: &Report, facts: &mut Declarations) -> Result<(), &'static str> {
    let snapshot = facts.header_bound;
    let Some(headers) = facts
        .must_use_contracts
        .as_deref_mut()
        .and_then(|owner| owner.headers.as_deref_mut())
        .map(|ledger| &mut ledger.local)
    else {
        return Ok(());
    };
    if InputBound::of(report) != snapshot {
        return Err("lowering report differs from its declarations' input graph");
    }
    if let Some(account) = &headers.lowering {
        // A later lowering of the same crate demands its own bodies.
        account.carried.set(None);
        return Ok(());
    }
    headers.lowering = Some(Account {
        steps: Cell::new(0),
        bytes: Cell::new(0),
        bound: bound(report),
        scope: Cell::new(None),
        carried: Cell::new(None),
    });
    Ok(())
}

fn account(facts: &Declarations) -> Option<&Account> {
    facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.local.lowering.as_ref())
}

/// Open a demand scope, unless no account is open or a scope already is, so a
/// nested scope leaves every charge to the enclosing one.
fn open_scope(account: Option<&Account>) -> Option<&Account> {
    let account = account.filter(|account| account.scope.get().is_none())?;
    account.scope.set(Some([0, 0]));
    Some(account)
}

/// Run `work` inside a demand scope and return what it charged there. The
/// scope closes before this returns, whatever `work` returns, so no later
/// charge can stay routed to the demand budget.
pub(super) fn scoped<T>(account: Option<&Account>, work: impl FnOnce() -> T) -> (T, [usize; 2]) {
    let opened = open_scope(account);
    let result = work();
    let charged = opened
        .and_then(|account| account.scope.take())
        .unwrap_or_default();
    (result, charged)
}

/// Run one demand traversal action with `facts` inside a demand scope.
pub(super) fn traverse<T>(
    facts: &mut Declarations,
    work: impl FnOnce(&mut Declarations) -> T,
) -> (T, [usize; 2]) {
    let opened = open_scope(account(facts)).is_some();
    let result = work(facts);
    let charged = account(facts)
        .filter(|_| opened)
        .and_then(|account| account.scope.take())
        .unwrap_or_default();
    (result, charged)
}

/// Mark proof indices from `first` on as demanded for emission, with the
/// bytes their traversal left on the demand budget.
pub(super) fn carry(facts: &Declarations, first: usize, bytes: usize) {
    if let Some(account) = account(facts) {
        account.carried.set(Some([first, bytes]));
    }
}

/// The carried first demanded index and demand bytes, if a traversal ran.
pub(super) fn carried(facts: &Declarations) -> Option<[usize; 2]> {
    account(facts)?.carried.get()
}

/// Emit proof `index` inside a demand scope when its traversal demanded it.
pub(super) fn emit<T>(
    facts: &Declarations,
    index: usize,
    work: impl FnOnce() -> T,
) -> (T, [usize; 2]) {
    let demanded = account(facts).filter(|account| {
        account
            .carried
            .get()
            .is_some_and(|[first, _]| index >= first)
    });
    scoped(demanded, work)
}

/// Emission is over: no later index is demanded by this traversal.
pub(super) fn settle(facts: &Declarations) {
    if let Some(account) = account(facts) {
        account.carried.set(None);
    }
}

/// Charge originals-sized storage to the crate's existing ledger, lowering's
/// account once it is open.
pub(super) fn pay_growth(
    facts: &Declarations,
    steps: usize,
    bytes: usize,
) -> Result<(), &'static str> {
    facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .map_or(Ok(()), |ledger| {
            Payments(&ledger.local).charge(steps, bytes)
        })
}

fn bound(report: &Report) -> InputBound {
    let (elements, _) = input_graph(report);
    let owners = body_owners(report);
    InputBound {
        work: retained::sum([
            elements.saturating_mul(WORK_PER_INPUT),
            owners.saturating_mul(WORK_PER_BODY_OWNER),
        ]),
        bytes: retained::sum([
            elements.saturating_mul(BYTES_PER_INPUT),
            owners.saturating_mul(BYTES_PER_BODY_OWNER),
        ]),
    }
}

/// Input items that can own a lowered body: function, constructor, constant
/// and static definitions, and every closure, inline constant, async block,
/// array length, repeat count and explicit discriminant written in the crate.
fn body_owners(report: &Report) -> usize {
    let definitions = report
        .resolution
        .definitions
        .iter()
        .filter(|definition| {
            matches!(
                definition.kind,
                DefKind::Function | DefKind::Constructor | DefKind::Const | DefKind::Static
            )
        })
        .count();
    let sites = report.package.units.iter().map(|unit| {
        let syntax = &unit.configured.ast;
        let nested = syntax
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.kind,
                    Kind::Closure { .. }
                        | Kind::Repeat { .. }
                        | Kind::Type {
                            flavor: TypeFlavor::Array,
                            ..
                        }
                        | Kind::Block {
                            flavor: BlockFlavor::Const
                                | BlockFlavor::Async
                                | BlockFlavor::AsyncMove,
                            ..
                        }
                )
            })
            .count();
        let discriminants = syntax
            .variants
            .iter()
            .filter(|variant| variant.discriminant.is_some())
            .count();
        retained::sum([nested, discriminants])
    });
    retained::sum(std::iter::once(definitions).chain(sites))
}

/// Lowering's steps, bytes and bound, once its account is open.
#[cfg(test)]
pub(crate) fn lowering_costs(facts: &Declarations) -> Option<[usize; 4]> {
    let account = account(facts)?;
    Some([
        account.steps.get(),
        account.bytes.get(),
        account.bound.work,
        account.bound.bytes,
    ])
}
#[cfg(test)]
#[path = "../../../tests/invariants/lowering_account.rs"]
mod tests;
