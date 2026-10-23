//! Precise lists select resolved parameter identities in declaration order.
use super::{Checker, DefId, Error, Kind, NodeKey, Target, regions};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    RegionId, Store, TypeId, arguments::Argument, bound_uses::Uses, parameters::clauses::Clause,
};

/// Only legacy free/inherent return opaques retain a written lifetime inventory.
/// Bound-local identities are filtered against the defining signature's slots.
#[derive(Default)]
pub(super) struct ExplicitLifetimes {
    pub names: HashSet<DefId>,
    pub regions: HashSet<RegionId>,
}

impl Checker<'_> {
    pub(super) fn legacy_opaque_captures(
        &self,
        key: NodeKey,
        owner: DefId,
    ) -> Result<Option<Box<ExplicitLifetimes>>, Error> {
        use crate::sema::associated::OwnerKind;
        let associated = &self.facts.associated;
        let trait_member = associated.trait_owner(owner).is_some()
            || associated
                .owner(owner)
                .and_then(|id| associated.get(id))
                .is_some_and(|owner| {
                    matches!(
                        owner.kind,
                        OwnerKind::Impl {
                            trait_type: Some(_),
                            ..
                        }
                    )
                });
        if trait_member {
            return Ok(None);
        }
        let span = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(key.node)
            .span;
        let edition = crate::resolve::unit::editions::introducer(key.file, span, |file| {
            self.report.package.units.get(file.0 as usize)
        })
        .map_err(|error| self.error(key, error.message))?;
        Ok((edition == crate::support::edition::Edition::Rust2021)
            .then(|| Box::new(ExplicitLifetimes::default())))
    }

    pub(super) fn opaque_captures(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
        precise: Option<NodeKey>,
        explicit: Option<&ExplicitLifetimes>,
    ) -> Result<Vec<Argument>, Error> {
        let binder = context
            .binder
            .ok_or_else(|| self.error(key, "missing opaque binder"))?;
        let mut captures = self
            .regions
            .capture_all(binder)
            .map_err(|e| self.error(key, e))?
            .into_iter()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>();
        let parameters = self
            .facts
            .generics
            .parameters_in_scope(owner)
            .filter(|p| !matches!(p.argument, Argument::Lifetime(_)))
            .copied()
            .collect::<Vec<_>>();
        captures.extend(parameters.iter().map(|p| p.argument));
        let Some(source) = precise else {
            if let Some(explicit) = explicit {
                let mut selected = explicit.regions.clone();
                for (name, region) in self
                    .regions
                    .bindings(binder)
                    .map_err(|e| self.error(key, e))?
                {
                    if explicit.names.contains(&name) {
                        selected.insert(region);
                    }
                }
                captures.retain(|argument| {
                    !matches!(argument, Argument::Lifetime(region) if !selected.contains(region))
                });
            }
            return Ok(captures);
        };
        if parameters.iter().any(|p| p.anonymous) {
            return Err(self.error(
                source,
                "precise capture cannot name an anonymous input parameter",
            ));
        }
        let Kind::PreciseCapture(list) = self.kind(source) else {
            return Err(self.error(source, "missing precise capture syntax"));
        };
        let nodes = self
            .report
            .package
            .unit(source.file)
            .configured
            .ast
            .list(list)
            .to_vec();
        let parameters = parameters
            .into_iter()
            .map(|p| (p.binding, p.argument))
            .collect::<HashMap<_, _>>();
        let mut selected = HashSet::default();
        let mut non_lifetime = false;
        for node in nodes {
            let node = NodeKey { node, ..source };
            let argument = if self.lifetime_argument(node) {
                if non_lifetime {
                    return Err(self.error(
                        node,
                        "captured lifetimes must precede type and const parameters",
                    ));
                }
                let region = self.region(node, Some(node.node), context)?;
                if region == RegionId::STATIC {
                    return Err(self.error(node, "precise capture requires a lifetime parameter"));
                }
                Argument::Lifetime(region)
            } else {
                non_lifetime = true;
                let Some(Target::Definition(id)) = self.facts.node(node).and_then(|n| n.reference)
                else {
                    return Err(self.error(node, "precise capture requires a resolved parameter"));
                };
                *parameters
                    .get(&id)
                    .ok_or_else(|| self.error(node, "precise capture names a non-parameter"))?
            };
            if !selected.insert(argument) {
                return Err(self.error(node, "precise capture repeats a parameter"));
            }
        }
        if parameters
            .values()
            .any(|argument| !selected.contains(argument))
        {
            return Err(self.error(source, "precise capture omits a type or const parameter"));
        }
        self.check_parent_captures(source, owner, &selected)?;
        captures.retain(|argument| selected.contains(argument));
        if captures.len() != selected.len() {
            return Err(self.error(source, "precise capture is outside the defining context"));
        }
        Ok(captures)
    }
    fn check_parent_captures(
        &self,
        source: NodeKey,
        owner: DefId,
        selected: &HashSet<Argument>,
    ) -> Result<(), Error> {
        if self.facts.associated.trait_owner(owner).is_some() {
            let parent = self
                .facts
                .associated
                .schema(owner)
                .ok_or_else(|| self.error(source, "missing trait capture schema"))?
                .captures;
            for slot in 0..parent {
                let slot = u16::try_from(slot)
                    .map_err(|_| self.error(source, "too many trait captures"))?;
                let region = RegionId::bound(0, slot).map_err(|e| self.error(source, e))?;
                if !selected.contains(&Argument::Lifetime(region)) {
                    return Err(self.error(
                        source,
                        "precise trait result capture omits a trait lifetime",
                    ));
                }
            }
        }
        Ok(())
    }
}

pub(super) fn check_promises(
    store: &Store,
    application: TypeId,
    bounds: &[Clause],
) -> Result<(), &'static str> {
    let mut captured = Uses::default();
    captured.collect(store, application)?;
    let mut required = Uses::default();
    for &bound in bounds {
        required.clause(store, bound)?;
    }
    if !required.indices.is_subset(&captured.indices) {
        return Err("opaque bound uses an excluded lifetime capture");
    }
    Ok(())
}
