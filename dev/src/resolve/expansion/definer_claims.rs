//! Authenticated defining attributes retain local alias paths until ordinary lookup.
use super::{Engine, Error, NodeKey, Report, Sources};
use crate::{
    expand::configured::Effect,
    lex::token::Punctuation,
    resolve::{
        bindings::policy::{self, Feature},
        index::{Context, State},
        lookup::{Lookup, LookupMode},
        model::{DefId, DefKind, Namespace},
        package::Package,
        paths::Saved,
    },
    support::{hash::HashMap, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Input, Style},
        item::ItemKind,
        token_tree::TreeRange,
    },
};

#[derive(Default)]
pub(super) struct Claims {
    pending: HashMap<NodeKey, Claim>,
    resolved: HashMap<DefId, Vec<DefId>>,
}
struct Claim {
    context: Context,
    effect: usize,
    offset: u32,
    image: std::sync::Arc<crate::support::source::OriginalImage>,
    attribute: crate::syntax::attribute::Attribute,
    paths: Vec<(bool, Saved)>,
}
impl Package {
    /// This request is a language gate as well as the builtin macro's API feature.
    pub(in crate::resolve) fn requested_named_opaque(&self) -> bool {
        self.source_features()
            .is_some_and(|features| features.has(Feature::TypeAliasImplTrait))
    }

    #[cfg(test)]
    pub(crate) fn named_opaque_usage(&self) -> Result<(usize, usize, usize), &'static str> {
        super::derive_lists::required_ledger(self, "missing named opaque payer")
            .map(super::super::bindings::panic_contracts::Contracts::usage)
    }

    pub(crate) fn pay_named_opaque(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.source_core()
            .ok_or("named opaques require the source-core profile")?
            .macros
            .charge(steps, bytes)
    }
    /// Only the declaration's own crate root can admit this language feature.
    pub(crate) fn require_named_opaque(&self, file: super::FileId) -> Result<(), Error> {
        let error = |message| Error::at(file, 0, message);
        self.pay_named_opaque(1, 0).map_err(error)?;
        let root = &self.modules[self.file_crate_root(file).0 as usize];
        let unit = self.unit(root.file);
        let attributes = unit.owner_indices[unit.configured.ast.root.get() as usize].map(|index| {
            unit.configured.owners[index.get() as usize - 1]
                .attributes
                .clone()
        });
        for index in attributes.into_iter().flatten() {
            let attribute = unit.configured.effects[index].attribute.attribute;
            self.pay_named_opaque(1 + (attribute.span.end - attribute.span.start) as usize, 0)
                .map_err(error)?;
        }
        if !policy::root_feature(unit, root.file, Feature::TypeAliasImplTrait)? {
            return Err(error(
                "type-alias impl Trait requires its own type_alias_impl_trait feature",
            ));
        }
        Ok(())
    }
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn define_opaque(&mut self, index: usize, definition: usize) -> Result<(), Error> {
        let call = &self.calls[index];
        if self.definitions[definition].node.is_none()
            || super::transported_macros::imported(&self.paths.state.facts, definition).is_some()
        {
            return Err(call
                .diagnostic("define_opaque requires its actual local source builtin declaration"));
        }
        self.checked_source_builtin(definition, super::Builtin::DefineOpaque)?;
        super::decorated_calls::validate(&self.loader.package, &self.paths.state.facts, call)?;
        let (visit, effect, offset) = (call.visit, call.capture as usize, call.offset);
        self.loader.package.require_named_opaque(visit.file)?;
        if visit.in_block
            || visit.foreign.is_some()
            || self.paths.state.facts.scopes[visit.path_scope.0 as usize].associated
        {
            return Err(self.calls[index]
                .diagnostic("define_opaque source profile requires a free function"));
        }
        let owner = item_owner(
            &self.loader.package,
            NodeKey {
                file: visit.file,
                node: visit.node,
            },
        )?;
        let ranges = self.opaque_path_ranges(index)?;
        self.loader
            .package
            .pay_named_opaque(
                ranges.len(),
                Growth::vector::<(bool, Saved)>(0, 0, ranges.len()).bytes,
            )
            .map_err(|message| self.calls[index].diagnostic(message))?;
        let mut paths = Vec::with_capacity(ranges.len());
        for range in ranges {
            let (absolute, path) = super::decorated_calls::path_range(
                &mut self.loader.package,
                visit.file,
                effect,
                range,
            )?;
            self.loader
                .package
                .pay_named_opaque(
                    path.len(),
                    path.len().saturating_mul(
                        size_of::<crate::resolve::names::NameId>()
                            + size_of::<crate::expand::marks::ItemContext>(),
                    ),
                )
                .map_err(|message| self.calls[index].diagnostic(message))?;
            paths.push((absolute, path.save()));
        }
        let unit = self.loader.package.unit(visit.file);
        let image = std::sync::Arc::clone(unit.source.original_image());
        let attribute = unit.configured.effects[effect].attribute.attribute;
        let claim = Claim {
            context: Context {
                module: visit.module,
                file: visit.file,
                scope: visit.path_scope,
                foreign: visit.foreign,
            },
            effect,
            offset,
            image,
            attribute,
            paths,
        };
        self.retain_opaque_claim(owner, claim)?;
        self.loader.package.units[visit.file.0 as usize]
            .configured
            .effects[effect]
            .attribute
            .effect = Effect::DefinedOpaque;
        self.derive_invocation(index, definition, visit.introduction)?;
        self.calls[index].selected = Some(definition);
        let tail = self.lint_call_tail(index)?;
        self.work.push(super::Work::Restore(self.head));
        self.head = self.calls[index].scope;
        self.push_visit_with_tail(visit, tail)?;
        self.scan()?;
        self.completed(index)
    }
    fn retain_opaque_claim(&mut self, owner: NodeKey, claim: Claim) -> Result<(), Error> {
        let old = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.opaques.as_deref());
        let (len, cap) = old.map_or((0, 0), |claims| {
            (claims.pending.len(), claims.pending.capacity())
        });
        let growth = Growth::map::<NodeKey, Claim>(
            len,
            cap,
            usize::from(old.is_none_or(|c| !c.pending.contains_key(&owner))),
        );
        self.loader
            .package
            .pay_named_opaque(
                growth.work,
                growth.bytes
                    + usize::from(old.is_none()) * size_of::<Claims>()
                    + usize::from(self.attribute_effects.is_none())
                        * size_of::<super::attribute_effects::Effects>(),
            )
            .map_err(|message| Error::at(owner.file, claim.offset, message))?;
        self.attribute_effects
            .get_or_insert_default()
            .opaques
            .get_or_insert_default()
            .pending
            .insert(owner, claim);
        Ok(())
    }
    fn opaque_path_ranges(&self, index: usize) -> Result<Vec<TreeRange>, Error> {
        let call = &self.calls[index];
        let unit = self.loader.package.unit(call.file);
        let effect = &unit.configured.effects[call.capture as usize];
        let attribute = effect.attribute.attribute;
        let Input::Delimited(group) = attribute.input else {
            return Err(call.diagnostic("define_opaque requires a list of type aliases"));
        };
        if attribute.style != Style::Outer || attribute.unsafe_wrapper {
            return Err(call.diagnostic("define_opaque requires an ordinary outer attribute"));
        }
        let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
        let range = forest
            .children(group)
            .ok_or_else(|| call.diagnostic("missing define_opaque list"))?;
        self.loader
            .package
            .pay_named_opaque(
                (range.end - range.start) as usize,
                Growth::vector::<TreeRange>(0, 0, (range.end - range.start) as usize).bytes,
            )
            .map_err(|message| call.diagnostic(message))?;
        let mut ranges = Vec::new();
        let mut start = range.start;
        for cursor in range.start..range.end {
            if forest.punctuation(cursor, Punctuation::Comma) {
                if start == cursor {
                    return Err(call.diagnostic("expected path to type alias"));
                }
                ranges.push(TreeRange { start, end: cursor });
                start = cursor + 1;
            }
        }
        if start < range.end {
            ranges.push(TreeRange {
                start,
                end: range.end,
            });
        }
        Ok(ranges)
    }
}
fn item_owner(package: &Package, mut key: NodeKey) -> Result<NodeKey, Error> {
    loop {
        package
            .pay_named_opaque(1, 0)
            .map_err(|message| Error::at(key.file, 0, message))?;
        match package.unit(key.file).configured.ast.node(key.node).kind {
            Kind::Attributed { value, .. } => key.node = value,
            Kind::Item(index)
                if matches!(
                    package.unit(key.file).configured.ast.items[index as usize].kind,
                    ItemKind::Function { body: Some(_), .. }
                ) =>
            {
                return Ok(key);
            }
            _ => {
                return Err(Error::at(
                    key.file,
                    0,
                    "define_opaque source profile requires a function with a body",
                ));
            }
        }
    }
}
impl Report {
    pub(crate) fn validate_named_opaque_definitions(&self) -> Result<(), Error> {
        if let Some(claims) = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.opaques.as_deref())
        {
            claims.validate(&self.package, &self.resolution)?;
        }
        Ok(())
    }

    pub(crate) fn named_opaque_definitions(&self) -> Option<&HashMap<DefId, Vec<DefId>>> {
        Some(
            &self
                .attribute_effects
                .as_deref()?
                .opaques
                .as_deref()?
                .resolved,
        )
    }
    pub(in crate::resolve) fn resolve_named_opaques(
        &mut self,
        state: &mut State,
    ) -> Result<(), Error> {
        let Some(mut claims) = self
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.opaques.take())
        else {
            return Ok(());
        };
        let result = claims.resolve(state, &mut self.package);
        self.attribute_effects.get_or_insert_default().opaques = Some(claims);
        result
    }
}
impl Claims {
    fn validate(
        &self,
        package: &Package,
        facts: &crate::resolve::model::Resolution,
    ) -> Result<(), Error> {
        package
            .pay_named_opaque(self.pending.capacity(), 0)
            .map_err(Error::unlocated)?;
        for (&owner, claim) in &self.pending {
            package
                .pay_named_opaque(1, 0)
                .map_err(|e| Error::at(owner.file, claim.offset, e))?;
            let error = |message| Error::at(owner.file, claim.offset, message);
            let unit = package.unit(owner.file);
            let actual = unit
                .configured
                .effects
                .get(claim.effect)
                .ok_or_else(|| error("define_opaque lost its configured effect"))?;
            let function = facts
                .item_definitions
                .get(&(claim.context.module, owner.node, Namespace::Value))
                .and_then(|owner| facts.definitions.get(owner.0 as usize));
            if !std::sync::Arc::ptr_eq(&claim.image, unit.source.original_image())
                || actual.attribute.effect != Effect::DefinedOpaque
                || actual.attribute.attribute != claim.attribute
                || function.is_none_or(|function| {
                    function.node != Some(owner) || function.kind != DefKind::Function
                })
                || facts
                    .scopes
                    .get(claim.context.scope.0 as usize)
                    .is_none_or(|scope| scope.module != claim.context.module)
            {
                return Err(error("define_opaque actual source owner or effect changed"));
            }
        }
        Ok(())
    }

    fn resolve(&mut self, state: &mut State, package: &mut Package) -> Result<(), Error> {
        package
            .pay_named_opaque(self.pending.capacity(), 0)
            .map_err(Error::unlocated)?;
        for (&owner, claim) in &self.pending {
            let error = |message| Error::at(owner.file, claim.offset, message);
            package.require_named_opaque(owner.file)?;
            if package.unit(owner.file).configured.effects[claim.effect]
                .attribute
                .effect
                != Effect::DefinedOpaque
            {
                return Err(error("define_opaque actual configured effect changed"));
            }
            let function = *state
                .facts
                .item_definitions
                .get(&(claim.context.module, owner.node, Namespace::Value))
                .ok_or_else(|| error("define_opaque lost its actual function"))?;
            let paths = Growth::vector::<DefId>(0, 0, claim.paths.len());
            let resolved =
                Growth::map::<DefId, Vec<DefId>>(self.resolved.len(), self.resolved.capacity(), 1);
            package
                .pay_named_opaque(
                    claim.paths.len() + paths.work + resolved.work,
                    paths.bytes + resolved.bytes,
                )
                .map_err(error)?;
            let mut aliases = Vec::with_capacity(claim.paths.len());
            for (absolute, path) in &claim.paths {
                let found = state.bind(package).lookup_path(
                    claim.context,
                    owner.node,
                    *absolute,
                    path.view(),
                    LookupMode::Use(Namespace::Type),
                    None,
                )?;
                let Lookup::Found(binding) = found else {
                    return Err(error("define_opaque requires an existing local type alias"));
                };
                let declaration = &state.facts.definitions[binding.target.0 as usize];
                let Some(key) = declaration.node.filter(|key| {
                    declaration.kind == DefKind::TypeAlias
                        && package.file_crate(key.file) == package.file_crate(owner.file)
                }) else {
                    return Err(error(
                        "define_opaque requires a type alias in its own source crate",
                    ));
                };
                package.pay_named_opaque(1, 0).map_err(error)?;
                if package
                    .unit(key.file)
                    .configured
                    .ast
                    .nodes
                    .get(key.node.get() as usize)
                    .is_none()
                {
                    return Err(error("define_opaque alias source changed"));
                }
                aliases.push(binding.target);
            }
            self.resolved.insert(function, aliases);
        }
        Ok(())
    }
}
