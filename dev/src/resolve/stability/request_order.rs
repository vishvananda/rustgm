//! Completed root requests reuse their vector; sorting precedes publication only.
use super::{Inventory, Library};
use crate::resolve::bindings::panic_contracts::Contracts;
use crate::support::crates::{MAX_BYTES, MAX_COMPONENTS, generics::canonical_order_paid};
use std::cmp::Ordering;
fn charge(steps: &mut usize, amount: usize) -> Result<(), &'static str> {
    *steps = steps.saturating_add(amount);
    if *steps > MAX_COMPONENTS {
        return Err("library request work budget exceeded");
    }
    Ok(())
}

fn compare(
    left: &str,
    right: &str,
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<Ordering, &'static str> {
    charge(1)?;
    for (left, right) in left.bytes().zip(right.bytes()) {
        charge(1)?;
        let order = left.cmp(&right);
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(left.len().cmp(&right.len()))
}
struct OrderingWork<'a> {
    steps: &'a mut usize,
    payer: Option<&'a Contracts>,
    transient: usize,
}
impl OrderingWork<'_> {
    fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.pay(amount, 0)
    }
    fn pay(&mut self, amount: usize, attempted: usize) -> Result<(), &'static str> {
        if let Some(payer) = self.payer {
            *self.steps = self.steps.saturating_add(amount);
            payer.inventory_order_charge(amount, attempted, self.transient)
        } else {
            charge(self.steps, amount)
        }
    }
}
impl Library {
    /// Non-type predicates use checked global requests only in ordinary crates.
    /// A staged library must prove each requirement in its actual item scope.
    pub(crate) fn predicate_requests(&self) -> Result<&[Box<str>], &'static str> {
        if self.inventory.is_none() {
            return Err("feature predicates require the completed request inventory");
        }
        Ok(if self.staged { &[] } else { &self.requests })
    }

    /// Undefined/conflicting requests and both declaration drafts were checked
    /// in source order. Only now may the same private vector be reordered.
    pub(in crate::resolve) fn prepare_request_order(
        &mut self,
        inventory: &mut Inventory,
        payer: Option<&Contracts>,
    ) -> Result<(), &'static str> {
        let count = self.requests.len();
        // The index plus the existing sorting kernel's two buffers coexist.
        // Later inverse/permutation storage is smaller; account both explicitly.
        let transient = count
            .saturating_mul(3 * size_of::<usize>())
            .saturating_add(3 * size_of::<Vec<usize>>());
        let mut work = OrderingWork {
            steps: &mut inventory.steps,
            payer,
            transient,
        };
        // Caller index, zeroed inverse, inverse mapping, final pass and swaps.
        // Publish no label permutation until every fallible operation is paid.
        work.pay(1 + count.saturating_mul(5), transient)?;
        if payer.is_none() && self.request_bytes.saturating_add(transient) > MAX_BYTES {
            return Err("library request ordering exceeds transient storage budget");
        }
        let mut order: Vec<_> = (0..count).collect();
        canonical_order_paid(
            &mut order,
            &mut work,
            &mut 0,
            OrderingWork::charge,
            |left, right, work| {
                compare(&self.requests[*left], &self.requests[*right], |amount| {
                    work.charge(amount)
                })
                .map(|order| order.then_with(|| left.cmp(right)))
            },
        )?;
        // The sorted index is now validated. Prepayment included initialization
        // and at most count-1 swaps, so moving labels has no fallible callback.
        let mut inverse = vec![0; count];
        for (target, &original) in order.iter().enumerate() {
            inverse[original] = target;
        }
        for index in 0..count {
            while inverse[index] != index {
                let other = inverse[index];
                self.requests.swap(index, other);
                inverse.swap(index, other);
            }
        }
        Ok(())
    }
    pub(crate) fn requested_feature(
        &self,
        feature: &crate::resolve::bindings::FeatureName<'_>,
        steps: &mut usize,
    ) -> Result<bool, &'static str> {
        charge(steps, 1)?;
        if self.inventory.is_none() {
            return Err("library requests lack their completed checked inventory");
        }
        let (mut start, mut end) = (0, self.requests.len());
        while start < end {
            let middle = start + (end - start) / 2;
            match compare(&self.requests[middle], feature.spelling(), |amount| {
                charge(steps, amount)
            })? {
                Ordering::Less => start = middle + 1,
                Ordering::Equal => return Ok(true),
                Ordering::Greater => end = middle,
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/request_payer.rs"]
mod tests;
