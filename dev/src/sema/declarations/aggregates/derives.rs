//! Derived implementations are checked under their generated parameter bounds.
use super::{Checker, DefId, Error, ItemId, NodeKey};
use crate::types::{
    TypeId,
    arguments::Argument,
    copy::{Copies, Properties},
    equality::Equality,
    parameters::{
        environment::Environment,
        predicates::{Bound, Predicate},
    },
};
use std::sync::Arc;

impl Checker<'_> {
    fn derive_environment(
        &self,
        item: ItemId,
        selected: Properties,
    ) -> Result<Option<Arc<Environment>>, &'static str> {
        let owner = DefId(item.0);
        let Some(environment) = self.facts.generics.environments.get(&owner) else {
            return Ok(None);
        };
        let bounds: Vec<_> = self
            .facts
            .generics
            .parameters
            .get(&owner)
            .into_iter()
            .flatten()
            .filter_map(|parameter| {
                let Argument::Type(ty) = parameter.argument else {
                    return None;
                };
                let bound = if selected.copy {
                    Bound::Copy
                } else if selected.clone_ {
                    Bound::Clone
                } else {
                    return None;
                };
                Some(Predicate { ty, bound })
            })
            .collect();
        Ok(Some(Arc::new(
            environment
                .as_ref()
                .clone()
                .with_predicates(&self.facts.types, &bounds)?,
        )))
    }
    fn derive_fields(&self, item: ItemId) -> impl Iterator<Item = TypeId> + '_ {
        self.facts
            .members
            .variants(item)
            .iter()
            .flat_map(|id| {
                let variant = self.facts.members.variants[id.0 as usize];
                &self.facts.members.fields[variant.start as usize..variant.end as usize]
            })
            .map(|field| field.ty)
    }
    pub(super) fn copy_obligations(&self) -> Result<(), Error> {
        for entry in &self.nominals.entries {
            let selected = self
                .facts
                .derives
                .get(&entry.item)
                .copied()
                .unwrap_or_default();
            let selected = selected.properties();
            // Each impl has its own generated environment: the Copy impl's
            // stronger bounds cannot justify the independent Clone impl.
            if selected.copy {
                self.derived_copy(entry.item, entry.key, true, entry.repr.packed.is_some())?;
            }
            if selected.clone_ {
                if entry.kind == crate::types::aggregate::Kind::Union && !selected.copy {
                    return Err(self.error(entry.key, "derived Clone for a union requires Copy"));
                }
                self.derived_copy(entry.item, entry.key, false, entry.repr.packed.is_some())?;
            }
        }
        Ok(())
    }
    fn derived_copy(
        &self,
        item: ItemId,
        key: NodeKey,
        copy: bool,
        packed: bool,
    ) -> Result<(), Error> {
        let generated = Properties {
            copy: copy || packed,
            clone_: true,
        };
        let generated = if self
            .facts
            .derives
            .get(&item)
            .is_some_and(|d| !d.has_argument_bounds())
        {
            Properties::default()
        } else {
            generated
        };
        let environment = self
            .derive_environment(item, generated)
            .map_err(|e| self.error(key, e))?;
        let mut query = Copies::for_derive(environment);
        for ty in self.derive_fields(item) {
            let properties = query
                .demand(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.derives,
                    ty,
                )
                .map_err(|e| self.error(key, e))?;
            let valid = if copy || packed {
                properties.copy
            } else {
                properties.clone_
            };
            if !valid {
                let message = if copy {
                    "derived Copy requires every field to implement Copy"
                } else if packed {
                    "derived Clone on a packed struct requires Copy fields"
                } else {
                    "derived Clone requires every field to implement Clone"
                };
                return Err(self.error(key, message));
            }
        }
        Ok(())
    }
    pub(super) fn equality_obligations(&self) -> Result<(), Error> {
        for entry in &self.nominals.entries {
            let Some(selected) = self.facts.equality_derives.get(&entry.item) else {
                continue;
            };
            if selected.partial_eq {
                self.derived_equality(entry.item, entry.key, false, entry.repr.packed.is_some())?;
            }
            if selected.eq {
                self.derived_equality(entry.item, entry.key, true, entry.repr.packed.is_some())?;
            }
        }
        Ok(())
    }
    fn derived_equality(
        &self,
        item: ItemId,
        key: NodeKey,
        eq: bool,
        packed: bool,
    ) -> Result<(), Error> {
        let environment = self
            .derive_environment(
                item,
                Properties {
                    copy: packed,
                    clone_: packed,
                },
            )
            .map_err(|e| self.error(key, e))?;
        let mut copies = Copies::for_derive(environment);
        let mut query = Equality::for_derive(
            item,
            crate::types::equality::Properties {
                partial_eq: true,
                eq,
            },
        );
        let mut copy = |ty| {
            copies
                .demand(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.derives,
                    ty,
                )
                .map(|p| p.copy)
        };
        for ty in self.derive_fields(item) {
            let properties = query
                .demand_with_copy(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.equality_derives,
                    ty,
                    &mut copy,
                )
                .map_err(|e| self.error(key, e))?;
            if (eq && !properties.eq) || (!eq && !properties.partial_eq) {
                let message = if eq {
                    "derived Eq requires every field to implement Eq"
                } else {
                    "derived PartialEq requires every field to implement PartialEq"
                };
                return Err(self.error(key, message));
            }
            if packed && !copy(ty).map_err(|e| self.error(key, e))? {
                return Err(self.error(
                    key,
                    "derived PartialEq on a packed struct requires Copy fields",
                ));
            }
        }
        Ok(())
    }
}
