//! Stable dependency ordering publishes each selected pattern scope exactly once.
use super::PreparedScope;
use crate::sema::{constants::PatternValue, model::Declarations};
use crate::support::hash::HashMap;
use std::{cmp::Reverse, collections::BinaryHeap};

pub(super) fn schedule(
    facts: &mut Declarations,
    scopes: &[Option<PreparedScope>],
) -> Result<Vec<usize>, &'static str> {
    charge(facts, scopes.len())?;
    let mut owners = HashMap::default();
    for (index, prepared) in scopes.iter().enumerate() {
        let prepared = prepared.as_ref().ok_or("missing pending pattern scope")?;
        if let Some(owner) = prepared.scope.values.owner {
            owners.entry(owner).or_insert_with(Vec::new).push(index);
        }
    }
    let mut incoming = vec![0_usize; scopes.len()];
    let mut users = vec![Vec::new(); scopes.len()];
    for (index, prepared) in scopes.iter().enumerate() {
        let prepared = prepared.as_ref().ok_or("missing pending pattern scope")?;
        let mut dependencies = Vec::new();
        for &(key, value, _) in &prepared.requests {
            charge(facts, 1)?;
            let PatternValue::Definition(owner) = value else {
                continue;
            };
            let owner = facts
                .constant_uses
                .get(&key)
                .map_or(owner, |value| value.definition);
            if let Some(indices) = owners.get(&owner) {
                charge(facts, indices.len())?;
                dependencies.extend(indices.iter().copied());
            }
        }
        dependencies.sort_unstable();
        dependencies.dedup();
        incoming[index] = dependencies.len();
        for dependency in dependencies {
            users[dependency].push(index);
        }
    }
    let mut ready = BinaryHeap::new();
    for (index, &count) in incoming.iter().enumerate() {
        if count == 0 {
            ready.push(Reverse(index));
        }
    }
    let mut order = Vec::with_capacity(scopes.len());
    while let Some(Reverse(index)) = ready.pop() {
        charge(facts, 1)?;
        order.push(index);
        for &user in &users[index] {
            charge(facts, 1)?;
            incoming[user] -= 1;
            if incoming[user] == 0 {
                ready.push(Reverse(user));
            }
        }
    }
    if order.len() != scopes.len() {
        return Err("cycle in constant pattern dependencies");
    }
    Ok(order)
}
fn charge(facts: &mut Declarations, amount: usize) -> Result<(), &'static str> {
    facts.patterns.completion_work = facts
        .patterns
        .completion_work
        .checked_add(amount)
        .filter(|&work| work <= 1_000_000)
        .ok_or("constant pattern dependency work budget exceeded")?;
    Ok(())
}
