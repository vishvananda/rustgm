//! Canonical Self requirements and literal colon bounds have different owners.
use super::{Declarations, Stage, Trait, g};
use crate::{
    resolve::unit::{FileId, NodeKey},
    sema::{declaration_inputs::Input, traits::Header},
    support::{hash::HashMap, retained},
    syntax::ast::NodeId,
    types::parameters::{clauses::Clause, predicates::Bound},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Prefix {
    pub canonical: u32,
    pub declared: u32,
}
/// Verify before emitting a trait row. Matching counts/locations are insufficient.
fn authenticate(
    facts: &Declarations,
    header: &Trait<'_>,
    canonical: &Header,
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<Prefix, &'static str> {
    charge(1)?;
    let owner = header
        .origin
        .source()
        .ok_or("source trait prefix has no actual source owner")?;
    let full = facts
        .generics
        .predicates
        .get(&header.definition)
        .map_or(&[][..], Vec::as_slice);
    let declared = facts.generics.declared_predicates(header.definition).len();
    if canonical.self_type != header.self_type
        || canonical.predicates.len() >= declared
        || declared > full.len()
    {
        return Err("source canonical/declared/full trait boundaries changed");
    }
    let mut count = 0usize;
    for input in header.inputs.iter() {
        charge(1)?;
        let expected = full
            .get(count)
            .ok_or("source trait inputs exceed their actual owner")?;
        if input.source.source() != Some(expected.source) || input.value != expected.value {
            return Err("source trait inputs changed their exact source/clause owner");
        }
        if let Some(expected) = canonical.predicates.get(count)
            && (input.source.source() != Some(expected.source) || input.value != expected.value)
        {
            return Err("source trait canonical prefix changed its exact source/clause");
        }
        count = count
            .checked_add(1)
            .ok_or("source trait input count overflow")?;
    }
    if count != full.len() {
        return Err("source trait inputs truncate their actual owner");
    }
    charge(1)?;
    let implicit = facts
        .generics
        .trait_self_bounds
        .get(&header.definition)
        .ok_or("source trait has no actual implicit own-trait premise")?;
    let actual = full
        .get(canonical.predicates.len())
        .ok_or("source canonical prefix lost its following own-trait premise")?;
    if actual.source != owner || actual.value != Clause::Predicate(*implicit) {
        return Err("source trait canonical prefix has a foreign implicit premise");
    }
    Ok(Prefix {
        canonical: u32::try_from(canonical.predicates.len())
            .map_err(|_| "source canonical prefix overflow")?,
        declared: u32::try_from(declared).map_err(|_| "source declared prefix overflow")?,
    })
}

pub(super) struct Inventory<'a> {
    pub prefix: Prefix,
    canonical: &'a Header,
    file: FileId,
    literal: &'a [NodeId],
    rows: HashMap<NodeKey, Option<u32>>,
}
impl<'a> Inventory<'a> {
    pub fn new(
        stage: &mut Stage,
        facts: &Declarations,
        header: &Trait<'_>,
        canonical: &'a Header,
        literal: &'a [NodeId],
    ) -> Result<Self, &'static str> {
        let prefix = authenticate(facts, header, canonical, |work| stage.charge(work))?;
        let source = header
            .origin
            .source()
            .ok_or("source trait has no actual source owner")?;
        if header.written.supertraits != literal.len() {
            return Err("source trait changed its borrowed literal bound inventory");
        }
        // Conservative aggregate reservation includes this transient index while
        // clause/type construction is live; no per-input ancestry graph is copied.
        stage.reserve(retained::new_map::<NodeKey, Option<u32>>(literal.len()))?;
        let mut rows = HashMap::default();
        for &node in literal {
            stage.charge(1)?;
            if rows
                .insert(
                    NodeKey {
                        file: source.file,
                        node,
                    },
                    None,
                )
                .is_some()
            {
                return Err("source trait repeats a literal bound node");
            }
        }
        Ok(Self {
            prefix,
            canonical,
            file: source.file,
            literal,
            rows,
        })
    }
    pub fn is_self(&self, facts: &Declarations, ordinal: usize) -> Result<bool, &'static str> {
        self.canonical
            .predicates
            .get(ordinal)
            .map_or(Ok(false), |predicate| {
                self.canonical.implies_self(&facts.types, predicate)
            })
    }
    pub fn record(
        &mut self,
        stage: &mut Stage,
        facts: &Declarations,
        ordinal: usize,
        input: Input,
        row: u32,
        is_self: bool,
    ) -> Result<(), &'static str> {
        stage.charge(1)?;
        if ordinal >= self.canonical.predicates.len() {
            return Ok(());
        }
        let source = input
            .source
            .source()
            .ok_or("source literal input has no source identity")?;
        let Some(slot) = self.rows.get_mut(&source) else {
            return Ok(());
        };
        let predicate = match input.value {
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                facts
                    .types
                    .quantified(id)
                    .ok_or("source literal clause has an unknown binder")?
                    .predicate
            }
        };
        if !is_self
            || predicate.ty != self.canonical.self_type
            || !(predicate.bound.trait_parts().is_some() || predicate.bound == Bound::Sized)
        {
            return Err("source literal supertrait has no exact typed Self/trait requirement");
        }
        if slot.replace(row).is_some() {
            return Err("source literal supertrait has ambiguous canonical clauses");
        }
        Ok(())
    }
    pub fn finish(self, stage: &mut Stage) -> Result<g::Range, &'static str> {
        stage.reserve(self.literal.len().saturating_mul(4 * size_of::<u32>()))?;
        let start = u32::try_from(stage.rows.clause_indices.len())
            .map_err(|_| "source literal indices overflow")?;
        for &node in self.literal {
            stage.charge(1)?;
            let row = self
                .rows
                .get(&NodeKey {
                    file: self.file,
                    node,
                })
                .copied()
                .flatten()
                .ok_or("source literal supertrait lacks its exact canonical clause")?;
            stage.rows.clause_indices.push(row);
        }
        Ok(g::Range {
            start,
            end: u32::try_from(stage.rows.clause_indices.len())
                .map_err(|_| "source literal indices overflow")?,
        })
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/supertrait_inventory.rs"]
mod tests;
