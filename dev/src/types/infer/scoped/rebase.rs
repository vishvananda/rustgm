//! A body inference variable cannot capture a surrounding callable's local slots.
use super::super::{Constructor, Context, Error, RegionId, Store, TermId, Value};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Rebasing {
    completed: HashMap<(TermId, u16, bool), TermId>,
}
impl Rebasing {
    pub fn apply(
        &mut self,
        context: &mut Context,
        store: &Store,
        root: TermId,
        amount: u16,
        inward: bool,
    ) -> Result<TermId, Error> {
        if amount == 0 {
            return Ok(root);
        }
        context.step()?;
        if let Value::Known(ty) = context.terms[root.0 as usize].value
            && store.escaping_binders(ty).ok_or(Error::InvalidIdentity)? == 0
        {
            return Ok(root);
        }
        let key = (root, amount, inward);
        if let Some(&value) = self.completed.get(&key) {
            return Ok(value);
        }
        let value = context.rebase_scoped(store, root, amount, inward)?;
        self.completed.insert(key, value);
        Ok(value)
    }
}

impl Context {
    fn rebase_scoped(
        &mut self,
        store: &Store,
        root: TermId,
        amount: u16,
        inward: bool,
    ) -> Result<TermId, Error> {
        let mut work = vec![((root, 0_u16), false)];
        let mut completed = HashMap::new();
        let mut children = Vec::new();
        while let Some(((id, depth), finish)) = work.pop() {
            self.step()?;
            let id = self.root(id)?;
            if completed.contains_key(&(id, depth)) {
                continue;
            }
            match self.terms[id.0 as usize].value {
                Value::Known(ty)
                    if store.escaping_binders(ty).ok_or(Error::InvalidIdentity)? <= depth =>
                {
                    completed.insert((id, depth), id);
                    continue;
                }
                Value::Variable(_) => {
                    completed.insert((id, depth), id);
                    continue;
                }
                _ => {}
            }
            let kind = self.materialize(store, id)?;
            let term = self.terms[id.0 as usize];
            let nested = depth
                .checked_add(u16::from(matches!(
                    kind,
                    Constructor::Function { .. } | Constructor::Dynamic { .. }
                )))
                .ok_or(Error::Overflow)?;
            if !finish {
                self.account_work((term.end - term.start) as usize)?;
                work.push(((id, depth), true));
                work.extend(
                    self.edges[term.start as usize..term.end as usize]
                        .iter()
                        .rev()
                        .map(|&child| ((child, nested), false)),
                );
                continue;
            }
            children.clear();
            for index in term.start..term.end {
                let child = self.root(self.edges[index as usize])?;
                children.push(
                    *completed
                        .get(&(child, nested))
                        .ok_or(Error::InvalidIdentity)?,
                );
            }
            let value =
                self.map_constructor_regions(store, kind, &children, depth, |region, depth| {
                    rebase(region, depth, amount, inward)
                })?;
            completed.insert((id, depth), value);
        }
        completed
            .get(&(root, 0))
            .copied()
            .ok_or(Error::InvalidIdentity)
    }
}

fn rebase(region: RegionId, nested: u16, amount: u16, inward: bool) -> Result<RegionId, Error> {
    let Some((depth, slot)) = region.bound_parts() else {
        return Ok(region);
    };
    if depth < nested {
        return Ok(region);
    }
    let depth = if inward {
        depth.checked_add(amount).ok_or(Error::Overflow)?
    } else {
        if depth - nested < amount {
            return Err(Error::Mismatch);
        }
        depth - amount
    };
    RegionId::bound(depth, slot).map_err(|_| Error::Overflow)
}
