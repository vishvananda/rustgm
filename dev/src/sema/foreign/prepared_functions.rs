//! Paid inert name preparation precedes one infallible callable publication.
use super::{Function, Item, Registry};
use crate::{
    resolve::model::DefId,
    support::{
        hash::HashSet,
        linkage::{Linkage, Naming},
        retained::{self, Payment},
    },
    syntax::item::Safety,
    types::TypeId,
};

pub(crate) struct PreparedFunctions {
    owner: std::ptr::NonNull<Registry>,
    names: usize,
    items: usize,
    rows: Vec<(DefId, Function)>,
}
impl Registry {
    pub(crate) fn prepare_source_functions(
        &mut self,
        inputs: &[(DefId, TypeId, Safety, &str)],
        pay: Payment<'_>,
    ) -> Result<PreparedFunctions, &'static str> {
        pay(
            inputs.len().saturating_mul(4).saturating_add(1),
            inputs
                .len()
                .saturating_mul(size_of::<(DefId, Function)>())
                .saturating_add(retained::new_map::<DefId, ()>(inputs.len())),
        )?;
        let mut keys = HashSet::with_capacity_and_hasher(
            inputs.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        let mut additional = 0_usize;
        for &(definition, _, _, name) in inputs {
            pay(name.len().saturating_add(1), 0)?;
            if name.is_empty()
                || name.len() > 4096
                || name.contains('\0')
                || !keys.insert(definition)
            {
                return Err("invalid or duplicate prepared exact source import");
            }
            if !self.items.contains_key(&definition) {
                additional += 1;
            }
        }
        if additional != 0 {
            pay(
                1,
                retained::new_map::<DefId, Item>(self.items.len().saturating_add(additional)),
            )?;
            self.items.reserve(additional);
        }
        let mut rows = Vec::with_capacity(inputs.len());
        for &(definition, signature, safety, name) in inputs {
            let id = self
                .names
                .intern_paid(name, &mut |growth| pay(growth.work, growth.bytes))?;
            let function = Function {
                signature,
                safety,
                naming: Naming::Exact(id),
                linkage: Linkage::Import,
            };
            if self
                .items
                .get(&definition)
                .is_some_and(|previous| *previous != Item::Function(function))
            {
                return Err("prepared exact source import conflicts with its actual declaration");
            }
            rows.push((definition, function));
        }
        Ok(PreparedFunctions {
            owner: std::ptr::NonNull::from(&mut *self),
            names: self.names.len(),
            items: self.items.len(),
            rows,
        })
    }
}
impl PreparedFunctions {
    pub(crate) fn require(&self, registry: &Registry) -> Result<(), &'static str> {
        if !std::ptr::eq(registry, self.owner.as_ptr())
            || registry.names.len() != self.names
            || registry.items.len() != self.items
        {
            return Err("prepared exact source imports changed their actual destination");
        }
        Ok(())
    }
    /// The private header transaction called require before its first publication;
    /// it performs no intervening foreign registry mutation or fallible operation.
    pub(crate) fn install(self, registry: &mut Registry) {
        for (definition, function) in self.rows {
            registry.items.insert(definition, Item::Function(function));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paid_names_fail_without_callable_publication_and_warm_growth_is_bounded()
    -> Result<(), Box<dyn std::error::Error>> {
        use std::cell::Cell;
        for count in [8_u32, 32, 128] {
            let mut registry = Registry::default();
            let names = (0..count)
                .map(|i| format!("symbol_{i}"))
                .collect::<Vec<_>>();
            let rows = names
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    Ok((
                        DefId(u32::try_from(i)?),
                        TypeId::UNIT,
                        Safety::Default,
                        s.as_str(),
                    ))
                })
                .collect::<Result<Vec<_>, std::num::TryFromIntError>>()?;
            for limit in [0, 3, 17, 50] {
                let mut rejected = Registry::default();
                let attempts = Cell::new(0);
                let result = rejected.prepare_source_functions(&rows, &|_, _| {
                    attempts.set(attempts.get() + 1);
                    if attempts.get() > limit {
                        Err("exhausted")
                    } else {
                        Ok(())
                    }
                });
                if result.is_err() {
                    assert!(rejected.items.is_empty());
                    assert!(attempts.get() > limit);
                }
            }
            let attempted = Cell::new(0);
            assert!(
                registry
                    .prepare_source_functions(&rows, &|steps, _| {
                        attempted.set(attempted.get() + steps);
                        Err("denied")
                    })
                    .is_err()
            );
            assert!(registry.items.is_empty() && registry.names.is_empty());
            assert!(attempted.get() > 0);
            let bytes = Cell::new(0_usize);
            let plan = registry.prepare_source_functions(&rows, &|_, growth| {
                bytes.set(bytes.get().saturating_add(growth));
                Ok(())
            })?;
            assert!(registry.items.is_empty());
            plan.require(&registry)?;
            let mut foreign = Registry::default();
            assert!(plan.require(&foreign).is_err());
            plan.install(&mut registry);
            assert_eq!(registry.items.len(), count as usize);
            let before = (registry.names.len(), registry.names.byte_len());
            let plan = registry.prepare_source_functions(&rows, &|_, _| Ok(()))?;
            plan.require(&registry)?;
            plan.install(&mut registry);
            assert_eq!((registry.names.len(), registry.names.byte_len()), before);
            assert!(bytes.get() < count as usize * 1024);
            let foreign_plan = foreign.prepare_source_functions(&rows, &|_, _| Ok(()))?;
            assert!(foreign_plan.require(&registry).is_err());
        }
        Ok(())
    }
}
