//! Source sizing defaults become actual scoped clauses, never blanket parameter evidence.
use super::{Checker, DefId, Error, GenericFlavor, NodeKey, Subject, Syntax, Written};
use crate::{
    support::bindings::LangItem, syntax::item::ItemKind, types::parameters::predicates::Bound,
};
use std::collections::HashMap;
const SIZED: u8 = 1;
const META: u8 = 2;
const POINTEE: u8 = 4;

impl Checker<'_> {
    pub(super) fn source_sized_bound(
        &self,
        key: NodeKey,
        definition: DefId,
    ) -> Result<bool, Error> {
        if !self
            .report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(|bindings| bindings.language(LangItem::Sized))
            .is_some_and(|declaration| declaration.definition == definition)
        {
            return Ok(false);
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
        let Some(bindings) = self.report.resolution.source_bindings.as_deref() else {
            return Ok(());
        };
        let meta = bindings
            .language(LangItem::MetaSized)
            .ok_or_else(|| self.error(key, "source sizing defaults require MetaSized"))?
            .definition;
        let pointee = bindings
            .language(LangItem::PointeeSized)
            .ok_or_else(|| self.error(key, "source sizing defaults require PointeeSized"))?
            .definition;
        let mut levels: HashMap<DefId, u8> = HashMap::new();
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
