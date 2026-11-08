//! Optional cold scratch accounting runs before allocation, relocation or copying.
//!
//! A paid graph may be parked across waits, so its byte charge is the capacity
//! it actually retains: each reservation pays exactly the slots it adds, as
//! [`crate::support::retained::vector`] and [`crate::support::retained::map`]
//! measure them, and is checked against that capacity afterwards. Work stays
//! the relocation estimate of [`Growth`].
use super::{Graph, Solver, Variances};
use crate::support::{
    hash::{HashMap, HashSet},
    retained::{Growth, Payment},
};
use std::hash::Hash;

/// The capacity a paid vector reservation produces: geometric, at least the
/// needed length and at least four slots (eight for bytes).
const fn vector_capacity(capacity: usize, needed: usize, element: usize) -> usize {
    let minimum = if element == 1 { 8 } else { 4 };
    let doubled = capacity.saturating_mul(2);
    let target = if doubled > needed { doubled } else { needed };
    if target > minimum { target } else { minimum }
}
/// The capacity the pinned standard hash table has after reserving room for
/// `needed` entries, for entries of at least four bytes: 3, 7 or 14 entries
/// below 15, then seven eighths of the next power of two buckets. Paid tables
/// never remove entries, so a reservation beyond the capacity always resizes.
fn table_capacity(capacity: usize, needed: usize) -> Option<usize> {
    let request = needed.max(capacity.saturating_add(1));
    let buckets = match request {
        0..4 => 4,
        4..8 => 8,
        8..15 => 16,
        _ => (request.checked_mul(8)? / 7).checked_next_power_of_two()?,
    };
    Some(if buckets <= 8 {
        buckets - 1
    } else {
        buckets / 8 * 7
    })
}

pub(super) fn vector<T>(
    pay: Option<Payment<'_>>,
    values: &mut Vec<T>,
    additional: usize,
) -> Result<(), &'static str> {
    let Some(pay) = pay else { return Ok(()) };
    let (length, capacity) = (values.len(), values.capacity());
    let needed = length.saturating_add(additional);
    if size_of::<T>() == 0 || needed <= capacity {
        return Ok(());
    }
    let target = vector_capacity(capacity, needed, size_of::<T>());
    let growth = Growth::vector::<T>(length, capacity, additional);
    pay(
        growth.work,
        (target - capacity).saturating_mul(size_of::<T>()),
    )?;
    values
        .try_reserve_exact(target - length)
        .map_err(|_| "variance scratch vector allocation failed")?;
    if values.capacity() != target {
        return Err("variance scratch vector capacity differs from its charge");
    }
    Ok(())
}
pub(super) fn map<K: Eq + Hash, V>(
    pay: Option<Payment<'_>>,
    values: &mut HashMap<K, V>,
    additional: usize,
) -> Result<(), &'static str> {
    const {
        assert!(
            size_of::<(K, V)>() >= 4,
            "paid variance tables hold entries of four bytes or more"
        );
    };
    let Some(pay) = pay else { return Ok(()) };
    let (length, capacity) = (values.len(), values.capacity());
    let needed = length.saturating_add(additional);
    if needed <= capacity {
        return Ok(());
    }
    let target =
        table_capacity(capacity, needed).ok_or("variance scratch map capacity overflow")?;
    let growth = Growth::map::<K, V>(length, capacity, additional);
    pay(
        growth.work,
        (target - capacity).saturating_mul(size_of::<(K, V)>()),
    )?;
    values
        .try_reserve(additional)
        .map_err(|_| "variance scratch map allocation failed")?;
    if values.capacity() != target {
        return Err("variance scratch map capacity differs from its charge");
    }
    Ok(())
}
pub(super) fn insert<K: Eq + Hash>(
    pay: Option<Payment<'_>>,
    values: &mut HashSet<K>,
    key: K,
) -> Result<bool, &'static str> {
    const {
        assert!(
            size_of::<K>() >= 4,
            "paid variance sets hold entries of four bytes or more"
        );
    };
    if let Some(pay) = pay {
        if values.contains(&key) {
            return Ok(false);
        }
        let (length, capacity) = (values.len(), values.capacity());
        if length >= capacity {
            let target = table_capacity(capacity, length.saturating_add(1))
                .ok_or("variance scratch seen capacity overflow")?;
            let growth = Growth::map::<K, ()>(length, capacity, 1);
            pay(
                growth.work,
                (target - capacity).saturating_mul(size_of::<K>()),
            )?;
            values
                .try_reserve(1)
                .map_err(|_| "variance scratch seen allocation failed")?;
            if values.capacity() != target {
                return Err("variance scratch seen capacity differs from its charge");
            }
        }
    }
    Ok(values.insert(key))
}
impl<'a> Graph<'a> {
    pub(super) fn paid(
        known: &'a Variances,
        declarations: &'a [Option<super::TypeId>],
        pay: Payment<'a>,
    ) -> Result<Self, &'static str> {
        // Do not use Graph::default: its uncharged Solver allocates literal nodes.
        Ok(Self {
            skip_external_nominals: false,
            independent: false,
            known: Some(known),
            declarations: Some(declarations),
            pending: Vec::new(),
            result: Variances::default(),
            slots: Vec::new(),
            variables: Vec::new(),
            solver: Solver::paid(pay)?,
            work: Vec::new(),
            seen: HashSet::default(),
        })
    }
    /// Keep a paid reached-closure graph across a wait, without its borrowed
    /// rows, declarations or payment.
    pub(super) fn park(self) -> Graph<'static> {
        Graph {
            skip_external_nominals: self.skip_external_nominals,
            independent: self.independent,
            known: None,
            declarations: None,
            pending: self.pending,
            result: self.result,
            slots: self.slots,
            variables: self.variables,
            solver: self.solver.park(),
            work: self.work,
            seen: self.seen,
        }
    }
    /// Resume a parked graph against the current rows and declarations. Rows
    /// completed during the wait are constants for nominals it has not reached.
    pub(super) fn resume(
        parked: Graph<'static>,
        known: &'a Variances,
        declarations: &'a [Option<super::TypeId>],
        pay: Payment<'a>,
    ) -> Self {
        Self {
            skip_external_nominals: parked.skip_external_nominals,
            independent: parked.independent,
            known: Some(known),
            declarations: Some(declarations),
            pending: parked.pending,
            result: parked.result,
            slots: parked.slots,
            variables: parked.variables,
            solver: Solver::resume(parked.solver, pay),
            work: parked.work,
            seen: parked.seen,
        }
    }
}
