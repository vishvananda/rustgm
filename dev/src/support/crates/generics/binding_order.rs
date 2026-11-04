//! Fallible canonical binding-key ordering pays comparisons before visiting bytes.
use crate::support::crates::generics::Work;
use std::cmp::Ordering;
pub(crate) fn sort<T>(
    bindings: &mut [T],
    work: &mut Work,
    swaps: &mut usize,
    compare: impl FnMut(&T, &T, &mut Work) -> Result<Ordering, &'static str>,
) -> Result<(), &'static str> {
    sort_paid(bindings, work, swaps, Work::charge, compare)
}

/// The caller's actual finite owner pays every attempted step before work.
/// Metadata retains its own limits through sort; source inventories use their
/// already configured cumulative payer without creating another work ledger.
pub(crate) fn sort_paid<T, B>(
    bindings: &mut [T],
    work: &mut B,
    swaps: &mut usize,
    mut charge: impl FnMut(&mut B, usize) -> Result<(), &'static str>,
    mut compare: impl FnMut(&T, &T, &mut B) -> Result<Ordering, &'static str>,
) -> Result<(), &'static str> {
    charge(work, 1)?;
    let count = bindings.len();
    if count > crate::support::crates::MAX_COMPONENTS {
        return Err("binding order exceeds its cardinality budget");
    }
    // Both buffer initializations, inverse permutation and outer permutation
    // pass are linear even when no comparison or swap is needed.
    charge(work, count.saturating_mul(4))?;
    let mut order: Vec<_> = (0..count).collect();
    let mut scratch = vec![0; count];
    let mut width = 1_usize;
    while width < count {
        charge(work, 1)?;
        let mut start = 0_usize;
        while start < count {
            charge(work, 1)?;
            let middle = start.saturating_add(width).min(count);
            let end = middle.saturating_add(width).min(count);
            let (mut left, mut right) = (start, middle);
            for slot in &mut scratch[start..end] {
                charge(work, 1)?;
                let take_left = right == end
                    || left < middle && {
                        compare(&bindings[order[left]], &bindings[order[right]], work)?
                            != Ordering::Greater
                    };
                *slot = if take_left {
                    let value = order[left];
                    left += 1;
                    value
                } else {
                    let value = order[right];
                    right += 1;
                    value
                };
            }
            start = end;
        }
        std::mem::swap(&mut order, &mut scratch);
        width = width.saturating_mul(2);
    }
    for (target, &original) in order.iter().enumerate() {
        scratch[original] = target;
    }
    for index in 0..count {
        while scratch[index] != index {
            charge(work, 1)?;
            let other = scratch[index];
            bindings.swap(index, other);
            scratch.swap(index, other);
            *swaps = swaps.saturating_add(1);
        }
    }
    for pair in bindings.windows(2) {
        if compare(&pair[0], &pair[1], work)? == Ordering::Equal {
            return Err("namespace has duplicate actual module/name/namespace keys");
        }
    }
    Ok(())
}
