//! Canonical Self requirements and literal colon bounds have different owners.
use super::{Declarations, Stage, Trait, g};
use crate::{
    resolve::unit::{FileId, NodeKey},
    sema::{declaration_inputs::Input, generics::Predicate, traits::Header},
    support::{hash::HashMap, retained},
    syntax::ast::NodeId,
    types::{
        ItemId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Prefix {
    pub canonical: u32,
    pub declared: u32,
}
/// The source declaration attaches deferred bindings after its implicit `Self`
/// premise. The wire header puts every canonical input before that premise and
/// retains the declared result obligations and elaborated assumptions after it.
#[derive(Clone, Copy)]
pub(super) struct InputOrder<'a> {
    canonical: &'a [Predicate],
    implicit: Input,
    results: &'a [(ItemId, Input)],
    elaborated: &'a [Predicate],
}
impl<'a> InputOrder<'a> {
    pub fn inputs(self) -> impl Iterator<Item = Input> + 'a {
        self.canonical
            .iter()
            .map(Input::from)
            .chain(std::iter::once(self.implicit))
            .chain(self.results.iter().map(|(_, input)| *input))
            .chain(self.elaborated.iter().map(Input::from))
    }
}

fn same(left: Input, right: Input) -> bool {
    left.source == right.source && left.value == right.value
}

/// The canonical and result inventories each keep their authored order and
/// must account for every declared clause exactly once, including its source.
fn authenticate<'a>(
    facts: &'a Declarations,
    header: &Trait<'_>,
    canonical: &'a Header,
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(Prefix, InputOrder<'a>), &'static str> {
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
    let canonical_inputs = &canonical.source_inputs()?.predicates;
    if canonical.self_type != header.self_type
        || canonical_inputs
            .len()
            .saturating_add(canonical.results.len())
            .saturating_add(1)
            != declared
        || declared > full.len()
        || header.inputs.len() != full.len()
    {
        return Err("source canonical/declared/full trait boundaries changed");
    }
    let implicit = facts
        .generics
        .trait_self_bounds
        .get(&header.definition)
        .ok_or("source trait has no actual implicit own-trait premise")?;
    let expected_implicit = Input {
        source: owner.into(),
        value: Clause::Predicate(*implicit),
    };
    let mut direct = canonical_inputs.iter().peekable();
    let mut results = canonical.results.iter().peekable();
    let mut has_implicit = false;
    for (ordinal, (input, actual)) in header.inputs.iter().zip(full).enumerate() {
        charge(1)?;
        if !same(input, actual.into()) {
            return Err("source trait inputs changed their exact source/clause owner");
        }
        if ordinal >= declared {
            continue;
        }
        if same(input, expected_implicit) && !has_implicit {
            has_implicit = true;
        } else if direct
            .next_if(|expected| same(input, (*expected).into()))
            .is_none()
            && results
                .next_if(|(_, expected)| same(input, *expected))
                .is_none()
        {
            return Err("source trait canonical or result input changed its exact source/clause");
        }
    }
    if !has_implicit || direct.peek().is_some() || results.peek().is_some() {
        return Err("source trait lost a canonical, result or implicit Self input");
    }
    Ok((
        Prefix {
            canonical: u32::try_from(canonical_inputs.len())
                .map_err(|_| "source canonical prefix overflow")?,
            declared: u32::try_from(declared).map_err(|_| "source declared prefix overflow")?,
        },
        InputOrder {
            canonical: canonical_inputs,
            implicit: expected_implicit,
            results: &canonical.results,
            elaborated: &full[declared..],
        },
    ))
}

pub(super) struct Inventory<'a> {
    pub prefix: Prefix,
    pub order: InputOrder<'a>,
    canonical: &'a Header,
    file: FileId,
    literal: &'a [NodeId],
    rows: HashMap<NodeKey, Option<u32>>,
}
impl<'a> Inventory<'a> {
    pub fn new(
        stage: &mut Stage,
        facts: &'a Declarations,
        header: &Trait<'_>,
        canonical: &'a Header,
        literal: &'a [NodeId],
    ) -> Result<Self, &'static str> {
        let (prefix, order) = authenticate(facts, header, canonical, |work| stage.charge(work))?;
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
            order,
            canonical,
            file: source.file,
            literal,
            rows,
        })
    }
    pub fn is_self(&self, facts: &Declarations, ordinal: usize) -> Result<bool, &'static str> {
        self.canonical
            .source_inputs()?
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
        if ordinal >= self.canonical.source_inputs()?.predicates.len() {
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
            Clause::UnstableFeature(_) => {
                return Err(
                    "non-type feature conditions require explicit trait metadata transport",
                );
            }
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                facts
                    .types
                    .quantified(id)
                    .ok_or("source literal clause has an unknown binder")?
                    .predicate
            }
        };
        let lifetime = matches!(predicate.bound, Bound::Outlives(_));
        if lifetime && !stage.declaration_policies() {
            return Err("source literal lifetime bounds require version fourteen");
        }
        if !is_self
            || predicate.ty != self.canonical.self_type
            || !(predicate.bound.trait_parts().is_some()
                || predicate.bound == Bound::Sized
                || lifetime)
        {
            return Err("source literal supertrait has no exact typed Self requirement");
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
