//! Source sizing defaults become actual scoped clauses, never blanket parameter evidence.
use super::{Checker, DefId, Error, GenericFlavor, NodeKey, Subject, Syntax, Written};
use crate::support::hash::HashMap;
use crate::{
    support::bindings::LangItem, syntax::item::ItemKind, types::parameters::predicates::Bound,
};
const SIZED: u8 = 1;
const META: u8 = 2;
const POINTEE: u8 = 4;

impl Checker<'_> {
    pub(super) fn source_sized_bound(
        &self,
        key: NodeKey,
        definition: DefId,
    ) -> Result<bool, Error> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(role) = catalog
            .role(LangItem::Sized)
            .map_err(|e| self.error(key, e))?
        else {
            return Ok(false);
        };
        if role.definition != definition {
            return Ok(false);
        }
        if matches!(
            role.origin,
            crate::resolve::body_origin::DeclarationOrigin::Imported(_)
        ) {
            self.facts
                .associated
                .require_imported_role(role)
                .map_err(|e| self.error(key, e))?;
        }
        if crate::sema::lifetimes::arguments(self.report, key, false)?
            .is_some_and(|list| list.start != list.end)
        {
            return Err(self.error(key, "source Sized does not accept generic arguments"));
        }
        Ok(true)
    }
    pub(super) fn source_sizing_defaults(
        &mut self,
        owner: DefId,
        key: NodeKey,
        parameters: &[NodeKey],
        syntax: &mut Vec<Syntax>,
    ) -> Result<(), Error> {
        let Some((meta, pointee)) = self.source_sizing_roles(key)? else {
            return Ok(());
        };
        let alias_self = self.source_alias_self(owner, key)?;
        let mut alias_level = 0;
        let mut levels: HashMap<DefId, u8> = HashMap::default();
        for bound in syntax.iter() {
            let binding = match bound.subject {
                Subject::Parameter(id) => Some(id),
                Subject::Type(key) => self.bound_parameter(key, false),
                Subject::Associated(_) => None,
            };
            let level = match bound.bound {
                Written::Trait(Bound::Sized) => SIZED,
                Written::User { definition, .. } if definition == meta => META,
                Written::User { definition, .. } if definition == pointee => POINTEE,
                _ => continue,
            };
            if let Some(binding) = binding {
                *levels.entry(binding).or_default() |= level;
                // TraitDef defaults inspect direct RHS bounds. A where-clause
                // has Subject::Type even when it names this same actual Self.
                if alias_self == Some(binding) && matches!(bound.subject, Subject::Parameter(_)) {
                    alias_level |= level;
                }
            }
        }
        for &key in parameters {
            if self.generic_syntax(key)?.flavor == GenericFlavor::Type {
                let binding = self.generic_binding(key)?;
                self.source_parameter_default(
                    binding,
                    key,
                    levels.get(&binding).copied().unwrap_or(0),
                    meta,
                    syntax,
                );
            }
        }
        if let Some(binding) = alias_self {
            if alias_level & SIZED == 0 {
                self.source_parameter_default(binding, key, alias_level, meta, syntax);
            } else {
                self.relaxed.remove(&binding);
            }
        } else if matches!(self.item(key)?.kind, ItemKind::Trait { auto_: false, .. })
            && owner != pointee
        {
            let binding = self
                .facts
                .associated
                .owner(owner)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| self.error(key, "source trait has no Self declaration"))?
                .self_definition;
            self.source_parameter_default(
                binding,
                key,
                levels.get(&binding).copied().unwrap_or(0),
                meta,
                syntax,
            );
        }
        Ok(())
    }
    /// Associated results inspect only their own decoded direct bounds.
    pub(in crate::sema::declarations) fn source_associated_default(
        &self,
        key: NodeKey,
        owner: DefId,
        relaxed: bool,
        required: &mut Vec<Bound>,
        syntax: &mut Vec<Syntax>,
    ) -> Result<(), Error> {
        if required.contains(&Bound::Sized) {
            return Ok(());
        }
        let roles = self.source_sizing_roles(key)?;
        if let Some((meta, pointee)) = roles {
            if syntax.iter().any(|bound| {
                matches!(bound.bound,
                Written::User { definition, .. } if definition == meta || definition == pointee)
            }) {
                return Ok(());
            }
            if relaxed {
                syntax.push(Syntax::associated(
                    key,
                    owner,
                    Written::SourceSizing(meta),
                    None,
                ));
                return Ok(());
            }
        }
        if !relaxed {
            required.push(Bound::Sized);
        }
        Ok(())
    }
    fn source_alias_self(&self, owner: DefId, key: NodeKey) -> Result<Option<DefId>, Error> {
        if !matches!(self.item(key)?.kind, ItemKind::TraitAlias { .. }) {
            return Ok(None);
        }
        let binding = self
            .alias_self(owner)
            .ok_or_else(|| self.error(key, "source alias has no actual Self declaration"))?;
        let actual = self
            .report
            .resolution
            .definitions
            .get(owner.0 as usize)
            .filter(|definition| {
                definition.kind == crate::resolve::model::DefKind::TraitAlias
                    && definition.node == Some(key)
            });
        if actual.is_none() || self.alias_self_schema(binding)?.is_none() {
            return Err(self.error(key, "source alias Self belongs to another declaration"));
        }
        Ok(Some(binding))
    }
    fn source_sizing_roles(&self, key: NodeKey) -> Result<Option<(DefId, DefId)>, Error> {
        let Some(bindings) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let meta = bindings
            .role(LangItem::MetaSized)
            .map_err(|e| self.error(key, e))?
            .ok_or_else(|| self.error(key, "source sizing defaults require MetaSized"))?;
        let pointee = bindings
            .role(LangItem::PointeeSized)
            .map_err(|e| self.error(key, e))?
            .ok_or_else(|| self.error(key, "source sizing defaults require PointeeSized"))?;
        for role in [meta, pointee] {
            if matches!(
                role.origin,
                crate::resolve::body_origin::DeclarationOrigin::Imported(_)
            ) {
                self.facts
                    .associated
                    .require_imported_role(role)
                    .map_err(|e| self.error(key, e))?;
            }
        }
        Ok(Some((meta.definition, pointee.definition)))
    }
    fn source_parameter_default(
        &mut self,
        binding: DefId,
        key: NodeKey,
        level: u8,
        meta: DefId,
        syntax: &mut Vec<Syntax>,
    ) {
        if level & (META | POINTEE) != 0 {
            if level & SIZED == 0 {
                self.relaxed.insert(binding);
            } else {
                self.relaxed.remove(&binding);
            }
            return;
        }
        syntax.push(Syntax {
            source: key,
            subject: Subject::Parameter(binding),
            bound: Written::SourceSizing(meta),
            scope: None,
        });
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/sizing_alias_contracts.rs"]
mod tests;
