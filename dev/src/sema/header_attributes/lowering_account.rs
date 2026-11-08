//! Lowering pays its own input-proportional account on the crate's ledger.
//!
//! Header analysis and lowering share one ledger, but not one bound: lowering
//! retains fixed proof and row slots per original body as well as work per
//! input element, so its account grants a per-element term and a per-body-owner
//! term, measured in `docs/parallel/lib193-header-budget.md`. Only storage the
//! originals size pays here; demanded instances pay the fixed demand budget by
//! their own output. Neither constant may be raised to absorb repeated or
//! superlinear work.
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
}
impl Account {
    pub(super) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
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
    if headers.lowering.is_some() {
        return Ok(());
    }
    headers.lowering = Some(Account {
        steps: Cell::new(0),
        bytes: Cell::new(0),
        bound: bound(report),
    });
    Ok(())
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
    let account = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.local.lowering.as_ref())?;
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
