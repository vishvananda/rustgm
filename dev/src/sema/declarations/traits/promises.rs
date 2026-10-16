//! Close builtin Self promises before projection identities acquire cached properties.
use super::super::generics::bounds::Written;
use super::super::{Checker, DefId, Error};
use crate::{
    sema::associated::OwnerKind,
    types::{copy::Properties, parameters::predicates::Bound},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Builtins {
    pub sized: bool,
    pub copies: Properties,
}
impl Builtins {
    fn bound(bound: Bound) -> Self {
        Self {
            sized: matches!(bound, Bound::Sized | Bound::Copy | Bound::Clone),
            copies: Properties {
                copy: bound == Bound::Copy,
                clone_: matches!(bound, Bound::Copy | Bound::Clone),
            },
        }
    }
    fn include(&mut self, other: Self) -> bool {
        let previous = *self;
        self.sized |= other.sized;
        self.copies.copy |= other.copies.copy;
        self.copies.clone_ |= other.copies.clone_;
        *self != previous
    }
}

impl Checker<'_> {
    /// The decoded bounds already retain resolved declaration identities. Their
    /// builtin Self promises are independent of application arguments, but remain
    /// conditional on proving the complete application through ordinary semantics.
    pub(super) fn inherited_builtins(&self) -> Result<HashMap<DefId, Builtins>, Error> {
        let mut promises = HashMap::new();
        let mut dependents: HashMap<DefId, Vec<DefId>> = HashMap::new();
        let mut work = Vec::new();
        let mut steps = 0_usize;
        for owner in self.facts.associated.owners() {
            let OwnerKind::Trait { definition, .. } = owner.kind else {
                continue;
            };
            steps =
                steps.saturating_add(1 + self.bound_syntax.get(&definition).map_or(0, Vec::len));
            if steps > 1_000_000 {
                return Err(self.error(
                    owner.source,
                    "associated builtin bound work budget exceeded",
                ));
            }
            let mut result = Builtins::default();
            for bound in self.declared_self_bounds(definition, owner.self_definition) {
                match bound {
                    Written::Trait(bound) => {
                        result.include(Builtins::bound(bound));
                    }
                    Written::User {
                        definition: parent, ..
                    } => {
                        dependents.entry(parent).or_default().push(definition);
                    }
                    Written::Outlives(_) => {}
                }
            }
            promises.insert(definition, result);
            if result != Builtins::default() {
                work.push(definition);
            }
        }
        // Each of three bits changes at most once per declaration. Cycles need
        // no special assumptions here; the ordinary trait gate still rejects them.
        while let Some(parent) = work.pop() {
            let inherited = promises[&parent];
            for &child in dependents.get(&parent).into_iter().flatten() {
                steps = steps.saturating_add(1);
                if steps > 1_000_000 {
                    let key = self.report.resolution.definitions[child.0 as usize]
                        .node
                        .ok_or_else(|| {
                            Error::at(crate::resolve::unit::FileId(0), 0, "missing trait source")
                        })?;
                    return Err(self.error(key, "associated builtin bound work budget exceeded"));
                }
                if promises.entry(child).or_default().include(inherited) {
                    work.push(child);
                }
            }
        }
        Ok(promises)
    }
}
