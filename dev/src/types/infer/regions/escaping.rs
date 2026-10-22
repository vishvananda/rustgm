//! Read-only binder demand over a partial graph, without creating type identities.
use super::super::{Constructor, Context, Error, RegionId, Store, TermId, Value, arguments::Slot};
use crate::support::hash::HashSet;
use crate::types::arguments::Argument;

impl Context {
    /// Inspect current inputs only. Unsolved holes contain no lifetime occurrence;
    /// callers must repeat the query after equations change those inputs.
    pub(crate) fn contains_escaping_regions(
        &mut self,
        store: &Store,
        root: TermId,
    ) -> Result<bool, Error> {
        let mut work = vec![(root, 0_u16)];
        let mut seen = HashSet::default();
        while let Some((id, depth)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if !seen.insert((id, depth)) {
                continue;
            }
            let term = self.terms[id.0 as usize];
            let kind = match term.value {
                Value::Variable(_) => continue,
                Value::Known(ty) => {
                    if store.escaping_binders(ty).ok_or(Error::InvalidIdentity)? > depth {
                        return Ok(true);
                    }
                    continue;
                }
                Value::Construct(kind) => kind,
            };
            if self.constructor_escapes(store, kind, depth)? {
                return Ok(true);
            }
            let nested = depth
                .checked_add(u16::from(self.binder_regions(kind)?.is_some()))
                .ok_or(Error::Overflow)?;
            self.account_work((term.end - term.start) as usize)?;
            work.extend(
                self.edges[term.start as usize..term.end as usize]
                    .iter()
                    .map(|&child| (child, nested)),
            );
        }
        Ok(false)
    }

    fn constructor_escapes(
        &mut self,
        store: &Store,
        kind: Constructor,
        mut depth: u16,
    ) -> Result<bool, Error> {
        match kind {
            Constructor::Reference { region, .. } => return Ok(escapes(region, depth)),
            Constructor::Dynamic { lifetimes, .. } => {
                if escapes(self.object_lifetimes(lifetimes)?.region, depth) {
                    return Ok(true);
                }
                depth = depth.checked_add(1).ok_or(Error::Overflow)?;
            }
            Constructor::Adt(_, args) | Constructor::FunctionItem(_, args) => {
                for argument in store.arguments(args).ok_or(Error::InvalidIdentity)? {
                    self.step()?;
                    let escaping = match argument {
                        Argument::Lifetime(region) => escapes(region, depth),
                        Argument::Type(ty) => {
                            store.escaping_binders(ty).ok_or(Error::InvalidIdentity)? > depth
                        }
                        Argument::Const(_) => false,
                    };
                    if escaping {
                        return Ok(true);
                    }
                }
            }
            _ => {}
        }
        if let Some((arguments, _)) = kind.application() {
            let count = self.slots(arguments)?.len();
            self.account_work(count)?;
            for slot in self.slots(arguments)? {
                if let Slot::Lifetime(region) = slot
                    && escapes(*region, depth)
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

fn escapes(region: RegionId, depth: u16) -> bool {
    region
        .bound_parts()
        .is_some_and(|(binder, _)| binder >= depth)
}

#[cfg(test)]
#[path = "../../../../tests/invariants/inferred_region_demand.rs"]
mod tests;
