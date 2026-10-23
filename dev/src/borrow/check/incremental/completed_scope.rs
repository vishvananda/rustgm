//! Complete actual proof groups lend indexed parent membership without cloning proofs.
use super::{Environment, Error, OwnedParent, Proved, ScopeOwner, Verifier};
use crate::support::hash::{BuildFxHasher, HashMap};
use crate::{
    body::{
        ir::{Body, BodyKey},
        regions::Contract,
    },
    borrow::{destruction::glue, regions, templates},
};
use std::hash::BuildHasher;

pub(crate) struct PairedProved {
    proved: Proved,
    lineage: regions::Lineage,
}
impl PairedProved {
    pub(crate) fn body(&self) -> &Body {
        self.proved.body()
    }
    pub(crate) fn input(&self) -> &Contract {
        self.lineage.input()
    }
    pub(crate) fn into_proved(self) -> Proved {
        self.proved
    }
}
impl Verifier {
    pub(crate) fn prove_paired<S: BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        templates: &templates::Checked<'_>,
        retained: regions::Retained,
        body: Body,
        catalog: Option<&mut glue::Catalog<'_>>,
    ) -> Result<PairedProved, Error> {
        let (lineage, regions) = retained.into_parts();
        let proved = self.prove(environment, templates, &regions, body, catalog, None)?;
        Ok(PairedProved { proved, lineage })
    }
}
pub(crate) struct CompleteGroup<'a> {
    seed: &'a regions::Lineage,
    rows: HashMap<BodyKey, &'a PairedProved>,
}
pub(crate) struct ParentMember<'a> {
    owner: &'a OwnedParent,
    lineage: &'a regions::Lineage,
}
impl<'a> CompleteGroup<'a> {
    pub(crate) fn new(
        mut rows: impl ExactSizeIterator<Item = &'a PairedProved>,
        bytes: usize,
        work: &mut usize,
    ) -> Result<Self, &'static str> {
        let count = rows.len();
        let retained = crate::support::retained::new_map::<BodyKey, &PairedProved>(count)
            .saturating_add(count.saturating_mul(size_of::<bool>()));
        if count > 1_000_000 || retained > bytes {
            return Err("actual completed group exceeds storage budget");
        }
        let first = rows.next().ok_or("actual completed group is empty")?;
        if count != first.lineage.len() {
            return Err("actual completed group has incomplete coverage");
        }
        let lineage = &first.lineage;
        let mut index = HashMap::with_capacity_and_hasher(count, BuildFxHasher::default());
        let mut seen = vec![false; count];
        for row in std::iter::once(first).chain(rows) {
            *work = work.saturating_add(1);
            let ordinal = row.lineage.ordinal();
            if !lineage.same_group(&row.lineage)
                || ordinal != index.len()
                || ordinal >= count
                || seen[ordinal]
                || row.input().key != row.body().key()
                || row.input().source != row.body().source
                || index.insert(row.body().key(), row).is_some()
            {
                return Err("actual completed group has foreign, duplicate or stale proof lineage");
            }
            seen[ordinal] = true;
        }
        Ok(Self {
            seed: lineage,
            rows: index,
        })
    }
    pub(crate) fn parent<'g>(
        &'g self,
        owner: &'g OwnedParent,
    ) -> Result<ParentMember<'g>, &'static str> {
        let row = owner.paired();
        if !self
            .rows
            .get(&row.body().key())
            .is_some_and(|actual| std::ptr::eq(*actual, row))
            || !self.seed.same_group(&row.lineage)
        {
            return Err("parent is not an actual member of this completed proof group");
        }
        Ok(ParentMember {
            owner,
            lineage: self.seed,
        })
    }
}
impl ParentMember<'_> {
    pub(crate) fn body(&self) -> &Body {
        self.owner.body()
    }
    pub(crate) fn scope_owner(&self) -> Result<ScopeOwner, &'static str> {
        if !self.lineage.same_group(&self.owner.paired().lineage) {
            return Err("parent membership changed its completed group lineage");
        }
        Ok(self.owner.scope_owner())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/actual_groups.rs"]
mod tests;
