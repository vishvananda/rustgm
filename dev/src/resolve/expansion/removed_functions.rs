//! Test removal retains authenticated ownership without an executed expansion frame.
use super::{Body, Builtin, CallGrammar, Engine, Error, NodeKey, Report, Sources, State};
use crate::{
    resolve::{
        model::{Namespace, ScopeId},
        package::ModuleId,
    },
    support::hash::HashMap,
    syntax::{
        ast::{Kind, NodeId},
        attribute::{Attribute, Input, Style},
        item::ItemKind,
    },
};

pub(crate) struct Removal {
    pub owner: NodeKey,
    pub declaration: NodeKey,
    pub effect: usize,
    attribute: Attribute,
    pub definition: usize,
    pub scope: ScopeId,
    pub module: ModuleId,
    pub duplicates: Vec<usize>,
}
#[derive(Default)]
pub(crate) struct Removals {
    pub entries: Vec<Removal>,
    owners: HashMap<NodeKey, usize>,
}
impl Removals {
    pub(crate) fn owner(&self, key: NodeKey) -> bool {
        self.owners.contains_key(&key)
    }
    pub(crate) fn get(&self, key: NodeKey) -> Option<&Removal> {
        self.owners
            .get(&key)
            .and_then(|&index| self.entries.get(index))
    }
    pub(crate) fn validate(&self, report: &Report) -> Result<(), Error> {
        for (index, removal) in self.entries.iter().enumerate() {
            if self.owners.get(&removal.owner) != Some(&index) {
                return Err(Error::at(
                    removal.owner.file,
                    0,
                    "removed Test owner index changed",
                ));
            }
            removal.validate_owner(report)?;
            removal.validate_role(report)?;
        }
        Ok(())
    }
}
impl Removal {
    fn validate_owner(&self, report: &Report) -> Result<(), Error> {
        let removal = self;
        let error = |message| Error::at(removal.owner.file, 0, message);
        let unit = report
            .package
            .units
            .get(removal.owner.file.0 as usize)
            .ok_or_else(|| error("removed Test source file is missing"))?;
        let owner = unit
            .owner_indices
            .get(removal.owner.node.get() as usize)
            .copied()
            .flatten()
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or_else(|| error("removed Test function lost its configured owner"))?;
        if removal.declaration.file != removal.owner.file
            || report
                .package
                .modules
                .get(removal.module.0 as usize)
                .is_none_or(|module| module.file != removal.owner.file)
            || owner.enabled
            || owner.node != removal.owner.node
            || !owner.attributes.contains(&removal.effect)
            || unit
                .retained_attributes(removal.effect..removal.effect + 1)
                .next()
                .is_none_or(|(attribute, _)| attribute != removal.attribute)
            || unit
                .configured
                .active
                .get(removal.owner.node.get() as usize)
                != Some(&false)
            || report
                .resolution
                .node_scopes
                .get(&(removal.module, removal.owner.node))
                != Some(&removal.scope)
            || report
                .resolution
                .scopes
                .get(removal.scope.0 as usize)
                .is_none_or(|scope| scope.module != removal.module)
            || report.resolution.item_definitions.contains_key(&(
                removal.module,
                removal.declaration.node,
                Namespace::Value,
            ))
            || function_declaration(unit, removal.owner.node).map_err(error)?
                != removal.declaration.node
            || removal.duplicates.iter().any(|effect| {
                !owner.attributes.contains(effect)
                    || *effect <= removal.effect
                    || unit
                        .retained_attributes(*effect..*effect + 1)
                        .next()
                        .is_none_or(|(attribute, forest)| {
                            attribute.name(forest, unit.source.text()) != Some("test")
                        })
            })
        {
            return Err(error("removed Test function actual ownership changed"));
        }
        Ok(())
    }
    fn validate_role(&self, report: &Report) -> Result<(), Error> {
        let removal = self;
        let error = |message| Error::at(removal.owner.file, 0, message);
        let definition = report
            .definitions
            .get(removal.definition)
            .filter(|definition| matches!(definition.body, Body::Builtin(Builtin::Test)))
            .ok_or_else(|| error("removed Test function selected role changed"))?;
        if let Some(key) = definition.node {
            let header = crate::resolve::bindings::panic_contracts::published(
                &report.package,
                key,
                definition.header.as_ref(),
                1,
            )?;
            if header.builtin != Some(Builtin::Test) {
                return Err(error("removed Test function source role changed"));
            }
        } else if report.package.uses_source_bindings() || definition.header.is_some() {
            return Err(error("removed Test function has no actual source role"));
        }
        Ok(())
    }
}
impl Report {
    pub(crate) fn removed_function_owner(&self, key: NodeKey) -> bool {
        self.removed_functions()
            .is_some_and(|removals| removals.owner(key))
    }
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn remove_test_function(
        &mut self,
        index: usize,
        definition: usize,
    ) -> Result<(), Error> {
        if let Some(error) = self.test_form_error(index)? {
            return Err(error);
        }
        let removal = self.checked_test_removal(index, definition)?;
        let owner = removal.owner;
        let removals = &mut self.attribute_effects.get_or_insert_default().removals;
        if removals.entries.len() >= 1_000_000 || removals.owners.contains_key(&owner) {
            return Err(Error::at(
                owner.file,
                0,
                "test removal owner storage budget exceeded or duplicated",
            ));
        }
        removals.owners.insert(owner, removals.entries.len());
        removals.entries.push(removal);
        self.loader.package.units[owner.file.0 as usize].remove_function_owner(owner.node);
        self.calls[index].selected = Some(definition);
        self.completed(index)?;
        self.calls[index].state = State::Removed;
        Ok(())
    }
    fn test_form_error(&mut self, index: usize) -> Result<Option<Error>, Error> {
        let call = &self.calls[index];
        if !matches!(call.grammar, CallGrammar::Attribute) {
            return Err(call.diagnostic("Test removal requires an actual attribute call"));
        }
        super::decorated_calls::validate(&self.loader.package, &self.paths.state.facts, call)?;
        let unit = self.loader.package.unit(call.file);
        let effect = call.capture as usize;
        let attribute = unit.configured.effects[effect].attribute.attribute;
        if attribute.input == Input::Empty
            && !attribute.unsafe_wrapper
            && attribute.style == Style::Outer
        {
            return Ok(None);
        }
        let primary = call.diagnostic("test attribute requires an outer word attribute");
        let owner = unit.owner_indices[call.visit.node.get() as usize]
            .and_then(|owner| unit.configured.owners.get(owner.get() as usize - 1))
            .ok_or_else(|| call.diagnostic("test form lost its actual owner"))?;
        let duplicate = unit
            .retained_indexed_attributes(owner.attributes.clone())
            .find(|(other, attribute, forest)| {
                *other > effect && attribute.name(forest, unit.source.text()) == Some("test")
            })
            .map(|(_, attribute, _)| attribute.span.start);
        let Some(offset) = duplicate else {
            return Ok(Some(primary));
        };
        let file = call.file;
        let level = self.lint_call_policy(index)?.duplicate_macro_attributes;
        let result = match level {
            crate::resolve::lint_policy::Level::Allow => primary,
            crate::resolve::lint_policy::Level::Warn => {
                primary.retain_warnings(vec![self.lint_warning(index, offset)?])
            }
            crate::resolve::lint_policy::Level::Deny
            | crate::resolve::lint_policy::Level::Forbid => {
                Error::at(file, offset, "duplicated test attribute")
            }
        };
        Ok(Some(result))
    }
    fn checked_test_removal(&self, index: usize, definition: usize) -> Result<Removal, Error> {
        let call = &self.calls[index];
        if !matches!(call.grammar, CallGrammar::Attribute) {
            return Err(call.diagnostic("Test removal requires an actual attribute call"));
        }
        super::decorated_calls::validate(&self.loader.package, &self.paths.state.facts, call)?;
        let owner = NodeKey {
            file: call.file,
            node: call.visit.node,
        };
        let unit = self.loader.package.unit(call.file);
        let effect = call.capture as usize;
        let attribute = unit.configured.effects[effect].attribute.attribute;
        if attribute.input != Input::Empty
            || attribute.unsafe_wrapper
            || attribute.style != Style::Outer
        {
            return Err(call.diagnostic("test attribute requires an outer word attribute"));
        }
        let node =
            function_declaration(unit, owner.node).map_err(|message| call.diagnostic(message))?;
        let configured = unit.owner_indices[owner.node.get() as usize]
            .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
            .ok_or_else(|| call.diagnostic("test function has no configured owner"))?;
        if let Some(source) = self.loader.package.source_core() {
            source
                .macros
                .charge(
                    1 + configured.attributes.len(),
                    4 * (size_of::<Removals>()
                        + size_of::<Removal>()
                        + configured.attributes.len() * size_of::<usize>()),
                )
                .map_err(|message| call.diagnostic(message))?;
        }
        let duplicates: Vec<_> = configured
            .attributes
            .clone()
            .filter(|&other| other > effect)
            .filter(|&other| {
                unit.retained_attributes(other..other + 1)
                    .next()
                    .is_some_and(|(attribute, forest)| {
                        attribute.name(forest, unit.source.text()) == Some("test")
                    })
            })
            .collect();
        Ok(Removal {
            owner,
            declaration: NodeKey { node, ..owner },
            effect,
            attribute,
            definition,
            scope: call.visit.path_scope,
            module: call.visit.module,
            duplicates,
        })
    }
}

fn function_declaration(
    unit: &crate::resolve::unit::Unit,
    mut node: NodeId,
) -> Result<NodeId, &'static str> {
    let mut remaining = unit.configured.ast.nodes.len();
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        remaining = remaining
            .checked_sub(1)
            .ok_or("test owner ancestry work budget exceeded")?;
        node = value;
    }
    if !matches!(unit.configured.ast.node(node).kind, Kind::Item(item) if matches!(unit.configured.ast.items[item as usize].kind, ItemKind::Function {..}))
    {
        return Err("test attribute requires a function owner");
    }
    Ok(node)
}
