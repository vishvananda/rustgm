//! Compiler policy admits actual primitive owners without weakening nominal ownership.
use super::{Checker, DefId, Error, NodeKey, Type};
use crate::{
    resolve::{model::DefKind, unit::FileId},
    sema::associated::OwnerKind,
    support::hash::{HashMap, HashSet},
    syntax::attribute::{Input, Style},
    types::{ItemId, TypeId},
};

impl Checker<'_> {
    /// Interpret each retained marker once, before reserving its actual impl.
    pub(super) fn incoherent_members(&self) -> Result<HashSet<DefId>, Error> {
        let mut members = HashSet::default();
        let mut definitions = None;
        let mut allowances = self.report.macro_allowances();
        let mut steps = 0_usize;
        for (index, unit) in self.report.package.units.iter().enumerate() {
            if unit.include_context.is_some() {
                continue;
            }
            let file = FileId(
                u32::try_from(index).map_err(|_| Error::unlocated("too many policy files"))?,
            );
            for owner in unit.configured.owners.iter().filter(|owner| owner.enabled) {
                for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
                    steps += 1;
                    if steps > 1_000_000 {
                        return Err(Error::at(
                            file,
                            attribute.span.start,
                            "inherent policy work budget exceeded",
                        ));
                    }
                    if attribute.name(forest, unit.source.text())
                        != Some("rustc_allow_incoherent_impl")
                    {
                        continue;
                    }
                    let key = self.unattributed(NodeKey {
                        file,
                        node: owner.node,
                    });
                    let error = |message| self.error(key, message);
                    if attribute.style != Style::Outer
                        || attribute.input != Input::Empty
                        || attribute.unsafe_wrapper
                    {
                        return Err(error(
                            "incoherent method permission requires an outer word attribute",
                        ));
                    }
                    crate::resolve::bindings::root_options::require_internal(
                        self.report,
                        &mut allowances,
                        file,
                        attribute,
                        forest,
                    )?;
                    if definitions.is_none() {
                        definitions = Some(self.incoherent_definitions()?);
                    }
                    let member = definitions
                        .as_ref()
                        .and_then(|ids| ids.get(&key))
                        .copied()
                        .ok_or_else(|| {
                            error("incoherent permission requires an actual inherent method")
                        })?;
                    let actual = &self.report.resolution.definitions[member.0 as usize];
                    let implementation = self
                        .facts
                        .associated
                        .owner(member)
                        .and_then(|owner| self.facts.associated.get(owner));
                    if actual.kind != DefKind::Function
                        || !implementation.is_some_and(|owner| {
                            matches!(
                                owner.kind,
                                OwnerKind::Impl {
                                    trait_type: None,
                                    ..
                                }
                            )
                        })
                    {
                        return Err(error(
                            "incoherent permission requires an actual inherent method",
                        ));
                    }
                    if !members.insert(member) {
                        return Err(error("duplicate incoherent method permission"));
                    }
                }
            }
        }
        Ok(members)
    }

    /// Definition nodes own items; expression bindings only own local values.
    /// Build this index once, and only when the crate uses the internal marker.
    fn incoherent_definitions(&self) -> Result<HashMap<NodeKey, DefId>, Error> {
        let mut identities = HashMap::default();
        for (index, definition) in self.report.resolution.definitions.iter().enumerate() {
            if definition.kind != DefKind::Function {
                continue;
            }
            let Some(node) = definition.node else {
                continue;
            };
            let key = self.unattributed(node);
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::unlocated("too many inherent definitions"))?,
            );
            if identities.insert(key, id).is_some() {
                return Err(self.error(key, "ambiguous incoherent method source owner"));
            }
        }
        Ok(identities)
    }

    pub(super) fn validate_inherent_target(
        &self,
        owner: DefId,
        source: NodeKey,
        ty: TypeId,
    ) -> Result<(), Error> {
        let kind = self
            .facts
            .types
            .kind(ty)
            .ok_or_else(|| self.error(source, "missing inherent target type"))?;
        if let Type::Adt(item, _) = kind {
            return self.require_local_inherent(source, item);
        }
        if !matches!(
            kind,
            Type::Primitive(_)
                | Type::Reference { .. }
                | Type::Pointer { .. }
                | Type::Tuple(_)
                | Type::Array { .. }
                | Type::ArrayTerm { .. }
                | Type::Slice(_)
                | Type::Function { .. }
                | Type::Never
        ) {
            return Err(self.error(
                source,
                "inherent impl requires a nominal or authorized primitive target",
            ));
        }
        if self.report.package.coherence_is_core()
            || self
                .inherent_headers
                .permitted(owner, self.report.package.requested_internal_attributes())
        {
            return Ok(());
        }
        Err(self.error(source, "inherent impl requires a local nominal type"))
    }

    fn require_local_inherent(&self, source: NodeKey, item: ItemId) -> Result<(), Error> {
        let actual = self
            .report
            .resolution
            .definitions
            .get(item.0 as usize)
            .ok_or_else(|| self.error(source, "missing inherent nominal declaration"))?;
        if self
            .report
            .resolution
            .library_types
            .identify(DefId(item.0))
            .is_some()
            || actual.node.is_none()
        {
            return Err(self.error(source, "inherent impl requires a local nominal type; foreign permissions require their wider profile"));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/structural_impls.rs"]
mod tests;
