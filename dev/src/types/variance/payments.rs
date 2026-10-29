//! Optional cold scratch accounting runs before allocation, relocation or copying.
use super::{Graph, Solver, Variances};
use crate::support::{
    hash::{HashMap, HashSet},
    retained::{Growth, Payment},
};
use std::hash::Hash;

pub(super) fn vector<T>(
    pay: Option<Payment<'_>>,
    values: &mut Vec<T>,
    additional: usize,
) -> Result<(), &'static str> {
    if let Some(pay) = pay {
        let growth = Growth::vector::<T>(values.len(), values.capacity(), additional);
        pay(growth.work, growth.bytes)?;
        values
            .try_reserve(additional)
            .map_err(|_| "variance scratch vector allocation failed")?;
    }
    Ok(())
}
pub(super) fn map<K: Eq + Hash, V>(
    pay: Option<Payment<'_>>,
    values: &mut HashMap<K, V>,
    additional: usize,
) -> Result<(), &'static str> {
    if let Some(pay) = pay {
        let growth = Growth::map::<K, V>(values.len(), values.capacity(), additional);
        pay(growth.work, growth.bytes)?;
        values
            .try_reserve(additional)
            .map_err(|_| "variance scratch map allocation failed")?;
    }
    Ok(())
}
pub(super) fn insert<K: Eq + Hash>(
    pay: Option<Payment<'_>>,
    values: &mut HashSet<K>,
    key: K,
) -> Result<bool, &'static str> {
    if let Some(pay) = pay {
        if values.contains(&key) {
            return Ok(false);
        }
        let growth = Growth::map::<K, ()>(values.len(), values.capacity(), 1);
        pay(growth.work, growth.bytes)?;
        values
            .try_reserve(1)
            .map_err(|_| "variance scratch seen allocation failed")?;
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
}
