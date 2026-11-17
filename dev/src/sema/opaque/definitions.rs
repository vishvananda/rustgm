//! Named declarations keep alias coordinates separate from their authorized bodies.
mod components;
#[cfg(test)]
#[path = "../../../tests/invariants/definer_authority.rs"]
mod tests;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    support::{hash::HashMap, retained::Growth},
    types::{Kind, TypeId, opaque::OpaqueId, substitute::Bound},
};

#[derive(Default)]
pub(crate) struct Named {
    pub(super) aliases: HashMap<DefId, Vec<OpaqueId>>,
    applications: HashMap<(DefId, OpaqueId), TypeId>,
    components: HashMap<TypeId, HashMap<OpaqueId, TypeId>>,
    definers: HashMap<OpaqueId, DefId>,
    pending: HashMap<OpaqueId, DefId>,
    previous: HashMap<OpaqueId, TypeId>,
    prepared: bool,
    image: Option<std::sync::Arc<crate::support::source::OriginalImage>>,
}
impl Declarations {
    pub(crate) fn register_named_opaque(
        &mut self,
        report: &Report,
        alias: DefId,
        id: OpaqueId,
    ) -> Result<(), &'static str> {
        let old = self.opaques.named.as_deref();
        if let Some(named) = old {
            named.authenticate(report)?;
        }

        let rows = old.and_then(|named| named.aliases.get(&alias));
        let growth = Growth::map::<DefId, Vec<OpaqueId>>(
            old.map_or(0, |n| n.aliases.len()),
            old.map_or(0, |n| n.aliases.capacity()),
            usize::from(rows.is_none()),
        );
        let values =
            Growth::vector::<OpaqueId>(rows.map_or(0, Vec::len), rows.map_or(0, Vec::capacity), 1);
        report.package.pay_named_opaque(
            1 + growth.work + values.work,
            growth.bytes + values.bytes + usize::from(old.is_none()) * size_of::<Named>(),
        )?;
        let named = self.opaques.named.get_or_insert_default();
        if named.image.is_none() {
            named.image = Some(std::sync::Arc::clone(
                report
                    .package
                    .unit(crate::resolve::unit::FileId(0))
                    .source
                    .original_image(),
            ));
        }
        named.aliases.entry(alias).or_default().push(id);
        Ok(())
    }
    pub(crate) fn prepare_named_opaques(&mut self, report: &Report) -> Result<(), Error> {
        if let Some(named) = self.opaques.named.as_deref() {
            named.authenticate(report).map_err(Error::unlocated)?;
        }

        if self
            .opaques
            .named
            .as_deref()
            .is_none_or(|named| !named.prepared)
        {
            report.validate_named_opaque_definitions()?;
        }
        let Some(mut named) = self.opaques.named.take() else {
            let has_aliases = if let Some(claims) = report.named_opaque_definitions() {
                report
                    .package
                    .pay_named_opaque(claims.capacity(), 0)
                    .map_err(Error::unlocated)?;
                claims.values().any(|aliases| !aliases.is_empty())
            } else {
                false
            };
            if has_aliases {
                return Err(Error::unlocated(
                    "define_opaque item does not contain any opaque types",
                ));
            }
            return Ok(());
        };
        let result = if named.prepared {
            Ok(())
        } else {
            named.prepare(report, self)
        };
        self.opaques.named = Some(named);
        result.map_err(Error::unlocated)
    }
    pub(crate) fn named_opaque_application(&self, owner: DefId, id: OpaqueId) -> Option<TypeId> {
        self.opaques
            .named
            .as_deref()?
            .applications
            .get(&(owner, id))
            .copied()
    }
    pub(crate) fn require_named_application(
        &self,
        owner: DefId,
        id: OpaqueId,
    ) -> Result<Option<TypeId>, &'static str> {
        let actual = self.named_opaque_application(owner, id);
        if self.is_named_opaque(id) && actual.is_none() {
            return Err("named opaque is not authorized in this defining body");
        }
        if let Some(actual) = actual {
            let declaration = self
                .opaques
                .declarations
                .get(&id)
                .ok_or("named opaque lost its actual declaration")?;
            declaration.require_current(self, id)?;
            declaration.source()?;
            if !matches!(self.types.kind(actual),Some(Kind::Opaque(actual_id,_)) if actual_id==id) {
                return Err("named opaque application changed its canonical identity");
            }
        }
        Ok(actual)
    }
    pub(crate) fn opaque_defining_owner(&self, id: OpaqueId) -> Option<DefId> {
        let named = self.opaques.named.as_deref();
        named
            .and_then(|n| n.pending.get(&id).or_else(|| n.definers.get(&id)))
            .copied()
            .or_else(|| self.opaques.declarations.get(&id).map(|d| d.owner))
    }
    pub(crate) fn is_named_opaque(&self, id: OpaqueId) -> bool {
        self.opaques
            .named
            .as_deref()
            .is_some_and(|n| n.definers.contains_key(&id))
    }
    /// Map the declaration's complete capture list to this body's actual output use.
    pub(crate) fn named_opaque_opening(
        &self,
        report: &Report,
        owner: DefId,
        id: OpaqueId,
    ) -> Result<Option<Bound>, &'static str> {
        let Some(actual) = self.require_named_application(owner, id)? else {
            if self.is_named_opaque(id) {
                return Err("named opaque is not authorized in this defining body");
            }
            return Ok(None);
        };
        let declaration = self
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing named opaque declaration")?;
        let Some(Kind::Opaque(actual_id, arguments)) = self.types.kind(actual) else {
            return Err("invalid named opaque body application");
        };
        let Some(Kind::Opaque(formal_id, formal)) = self.types.kind(declaration.application) else {
            return Err("invalid named declaration application");
        };
        if actual_id != id || formal_id != id {
            return Err("named opaque opening changed its canonical identity");
        }
        Bound::captured_arguments_charged(&self.types, formal, arguments, &|steps, bytes| {
            report.package.pay_named_opaque(steps, bytes)
        })
        .map(Some)
    }
    /// Hidden types reach this only after the body's universal regions are closed.
    pub(crate) fn close_named_hidden(
        &mut self,
        report: &Report,
        owner: DefId,
        id: OpaqueId,
        hidden: TypeId,
    ) -> Result<TypeId, &'static str> {
        let Some(actual) = self.require_named_application(owner, id)? else {
            if self.is_named_opaque(id) {
                return Err("named opaque is not authorized in this defining body");
            }
            return Ok(hidden);
        };
        let declaration = self
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing named opaque declaration")?;
        let (Some(Kind::Opaque(formal_id, formal)), Some(Kind::Opaque(actual_id, arguments))) = (
            self.types.kind(declaration.application),
            self.types.kind(actual),
        ) else {
            return Err("invalid named opaque capture mapping");
        };
        if actual_id != id || formal_id != id {
            return Err("named opaque closing changed its canonical identity");
        }
        let mut reverse =
            Bound::captured_arguments_charged(&self.types, arguments, formal, &|steps, bytes| {
                report.package.pay_named_opaque(steps, bytes)
            })?;
        let hidden = reverse.apply_charged(&mut self.types, hidden, &|steps, bytes| {
            report.package.pay_named_opaque(steps, bytes)
        })?;
        components::require_alias_parameters(report, &self.types, hidden, declaration.owner)?;
        Ok(hidden)
    }
    /// Retain a prior canonical representation while this body's region proof runs.
    pub(crate) fn stage_named_hidden(
        &mut self,
        report: &Report,
        owner: DefId,
        id: OpaqueId,
    ) -> Result<(), &'static str> {
        if self.named_opaque_application(owner, id).is_none() {
            return Err("named opaque is not authorized in this defining body");
        }
        let previous = self.opaques.declarations.get(&id).and_then(|d| d.hidden);
        let named = self
            .opaques
            .named
            .as_mut()
            .ok_or("missing named opaque state")?;
        if named.pending.contains_key(&id) {
            return Err("named opaque already has an unfinished defining body");
        }
        let mut growth =
            Growth::map::<OpaqueId, DefId>(named.pending.len(), named.pending.capacity(), 1);
        growth.include(Growth::map::<OpaqueId, TypeId>(
            named.previous.len(),
            named.previous.capacity(),
            usize::from(previous.is_some()),
        ));
        report
            .package
            .pay_named_opaque(1 + growth.work, growth.bytes)?;
        named.pending.insert(id, owner);
        if let Some(previous) = previous {
            named.previous.insert(id, previous);
        }
        Ok(())
    }
    pub(crate) fn compare_named_hidden(
        &mut self,
        id: OpaqueId,
        hidden: TypeId,
    ) -> Result<(), &'static str> {
        let Some(named) = self.opaques.named.as_deref_mut() else {
            return Ok(());
        };
        if !named.definers.contains_key(&id) {
            return Ok(());
        }
        let old = named
            .previous
            .get(&id)
            .copied()
            .or_else(|| self.opaques.declarations.get(&id).and_then(|d| d.hidden));
        if old.is_some_and(|old| old != hidden) {
            return Err("named opaque concrete type differs from its previous defining body");
        }
        named.pending.remove(&id);
        named.previous.remove(&id);
        Ok(())
    }
}
impl Named {
    fn authenticate(&self, report: &Report) -> Result<(), &'static str> {
        if self.image.as_ref().is_none_or(|image| {
            !std::sync::Arc::ptr_eq(
                image,
                report
                    .package
                    .unit(crate::resolve::unit::FileId(0))
                    .source
                    .original_image(),
            )
        }) {
            return Err("named opaque facts belong to a different source arena");
        }
        Ok(())
    }

    fn prepare(&mut self, report: &Report, facts: &mut Declarations) -> Result<(), &'static str> {
        if let Some(claims) = report.named_opaque_definitions() {
            report.package.pay_named_opaque(claims.capacity(), 0)?;
            for (&owner, aliases) in claims {
                self.prepare_body(report, facts, owner, aliases)?;
            }
        }
        report
            .package
            .pay_named_opaque(self.aliases.capacity(), 0)?;
        for ids in self.aliases.values() {
            for id in ids {
                report.package.pay_named_opaque(1, 0)?;
                if !self.definers.contains_key(id) {
                    return Err("named opaque alias has no authorized defining body");
                }
            }
        }
        self.prepared = true;
        Ok(())
    }
    fn prepare_components(
        &mut self,
        report: &Report,
        store: &crate::types::Store,
        root: TypeId,
    ) -> Result<(), &'static str> {
        report.package.pay_named_opaque(1, 0)?;
        if self.components.contains_key(&root) {
            return Ok(());
        }
        let growth = Growth::map::<TypeId, HashMap<OpaqueId, TypeId>>(
            self.components.len(),
            self.components.capacity(),
            1,
        );
        report.package.pay_named_opaque(growth.work, growth.bytes)?;
        let components = components::applications(report, store, root)?;
        self.components.insert(root, components);
        Ok(())
    }
    fn prepare_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        owner: DefId,
        aliases: &[DefId],
    ) -> Result<(), &'static str> {
        if aliases.is_empty() {
            return Ok(());
        }
        let (function_node, result) = defining_output(report, facts, owner)?;
        self.prepare_components(report, &facts.types, result)?;
        for alias in aliases {
            report.package.pay_named_opaque(1, 0)?;
            let ty = facts
                .definitions
                .get(alias.0 as usize)
                .copied()
                .flatten()
                .ok_or("define_opaque selected alias has no completed type")?;
            self.prepare_components(report, &facts.types, ty)?;
            let selected = &self.components[&ty];
            if selected.is_empty() {
                return Err("define_opaque item does not contain any opaque types");
            }
            report.package.pay_named_opaque(selected.capacity(), 0)?;
            for &id in selected.keys() {
                let declaration = facts
                    .opaques
                    .declarations
                    .get(&id)
                    .ok_or("selected alias has no opaque declaration")?;
                let source = declaration.source()?;
                let actual_alias = report
                    .resolution
                    .definitions
                    .get(declaration.owner.0 as usize)
                    .ok_or("selected opaque has no actual alias")?;
                if actual_alias.kind != crate::resolve::model::DefKind::TypeAlias
                    || actual_alias.node.is_none_or(|node| {
                        report.package.file_crate(node.file)
                            != report.package.file_crate(function_node.file)
                    })
                    || report.package.file_crate(source.file)
                        != report.package.file_crate(function_node.file)
                {
                    return Err("selected opaque belongs to a foreign alias declaration");
                }

                let actual = *self.components[&result].get(&id).ok_or(
                    "define_opaque body does not constrain the selected opaque in its output",
                )?;
                components::defining_arguments(report, &facts.types, actual, owner)?;
                let growth = Growth::map::<(DefId, OpaqueId), TypeId>(
                    self.applications.len(),
                    self.applications.capacity(),
                    1,
                );
                let reverse = Growth::map::<OpaqueId, DefId>(
                    self.definers.len(),
                    self.definers.capacity(),
                    usize::from(!self.definers.contains_key(&id)),
                );
                let rows = facts.opaques.owners.get(&owner);
                let owners = Growth::map::<DefId, Vec<OpaqueId>>(
                    facts.opaques.owners.len(),
                    facts.opaques.owners.capacity(),
                    usize::from(rows.is_none()),
                );
                let values = Growth::vector::<OpaqueId>(
                    rows.map_or(0, Vec::len),
                    rows.map_or(0, Vec::capacity),
                    1,
                );
                report.package.pay_named_opaque(
                    1 + growth.work + reverse.work + owners.work + values.work,
                    growth.bytes + reverse.bytes + owners.bytes + values.bytes,
                )?;
                if self.applications.insert((owner, id), actual).is_none() {
                    facts.opaques.owners.entry(owner).or_default().push(id);
                }
                self.definers
                    .entry(id)
                    .and_modify(|previous| {
                        if owner.0 < previous.0 {
                            *previous = owner;
                        }
                    })
                    .or_insert(owner);
            }
        }
        Ok(())
    }
}

fn defining_output(
    report: &Report,
    facts: &Declarations,
    owner: DefId,
) -> Result<(crate::resolve::unit::NodeKey, TypeId), &'static str> {
    let function = facts
        .function_indices
        .get(&owner)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("define_opaque has no actual defining function")?;
    let declaration = report
        .resolution
        .definitions
        .get(owner.0 as usize)
        .ok_or("named defining function has no declaration")?;
    if declaration.node != Some(function.node)
        || declaration.kind != crate::resolve::model::DefKind::Function
    {
        return Err("named defining function changed its actual source declaration");
    }
    if function.body.is_none() || facts.associated.owner(owner).is_some() {
        return Err("define_opaque requires its source body");
    }
    let Some(Kind::Function { result, .. }) = facts.types.kind(function.signature) else {
        return Err("define_opaque function signature changed");
    };
    Ok((function.node, result))
}
