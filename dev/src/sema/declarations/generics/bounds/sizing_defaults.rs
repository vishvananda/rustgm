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
        let Some(bindings) = self.facts.source_bindings.as_deref() else {
            return Ok(());
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
        let (meta, pointee) = (meta.definition, pointee.definition);
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
            }
        }
        for &key in parameters {
            if self.generic_syntax(key)?.flavor == GenericFlavor::Type {
                let binding = self.generic_binding(key)?;
                self.source_parameter_default(binding, key, &levels, meta, syntax);
            }
        }
        if matches!(self.item(key)?.kind, ItemKind::Trait { .. }) && owner != pointee {
            let binding = self
                .facts
                .associated
                .owner(owner)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| self.error(key, "source trait has no Self declaration"))?
                .self_definition;
            self.source_parameter_default(binding, key, &levels, meta, syntax);
        }
        Ok(())
    }
    fn source_parameter_default(
        &mut self,
        binding: DefId,
        key: NodeKey,
        levels: &HashMap<DefId, u8>,
        meta: DefId,
        syntax: &mut Vec<Syntax>,
    ) {
        let level = levels.get(&binding).copied().unwrap_or(0);
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
