//! Source-demanded inference prepays collection growth through its local meter.
use super::Budget;
use crate::support::{
    hash::{HashMap, HashSet},
    retained::Growth,
};
use std::hash::Hash;

impl Budget<'_> {
    /// Retained clears cost at most four times their touched set. A table left
    /// sparse by a prior wide owner is paid and retired once, rather than cleared
    /// at its old capacity for every subsequent small owner.
    pub(super) fn clear_set<T>(&mut self, values: &mut HashSet<T>) -> Result<(), &'static str> {
        if self.1.is_some() {
            self.work(values.capacity())?;
            if values.capacity() > values.len().saturating_mul(4).saturating_add(16) {
                *values = HashSet::default();
                return Ok(());
            }
        }
        values.clear();
        Ok(())
    }
    pub(super) fn push<T>(&mut self, values: &mut Vec<T>, value: T) -> Result<(), &'static str> {
        self.vector(values, 1)?;
        values.push(value);
        Ok(())
    }
    fn growth(&mut self, growth: Growth) -> Result<(), &'static str> {
        if let Some(pay) = self.1 {
            self.work(growth.work)?;
            pay(0, growth.bytes)?;
        }
        Ok(())
    }
    pub(super) fn vector<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), &'static str> {
        if self.1.is_some() {
            self.growth(Growth::vector::<T>(
                values.len(),
                values.capacity(),
                additional,
            ))?;
            values
                .try_reserve(additional)
                .map_err(|_| "outlives scratch allocation failed")?;
        }
        Ok(())
    }
    pub(super) fn map<K: Eq + Hash, V>(
        &mut self,
        values: &mut HashMap<K, V>,
        additional: usize,
    ) -> Result<(), &'static str> {
        if self.1.is_some() {
            self.growth(Growth::map::<K, V>(
                values.len(),
                values.capacity(),
                additional,
            ))?;
            values
                .try_reserve(additional)
                .map_err(|_| "outlives scratch allocation failed")?;
        }
        Ok(())
    }
    pub(super) fn insert<T: Eq + Hash>(
        &mut self,
        values: &mut HashSet<T>,
        value: T,
    ) -> Result<bool, &'static str> {
        if self.1.is_some() {
            self.charge()?;
            if values.contains(&value) {
                return Ok(false);
            }
            self.growth(Growth::map::<T, ()>(values.len(), values.capacity(), 1))?;
            values
                .try_reserve(1)
                .map_err(|_| "outlives scratch allocation failed")?;
        }
        Ok(values.insert(value))
    }
}
