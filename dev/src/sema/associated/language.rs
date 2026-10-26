//! Typed language contracts have no source owner or generated AST declaration.
use super::{DefId, Inventory, Schema};
use crate::support::call_traits::CallTrait;
use crate::support::hash::HashMap;
use crate::support::language_hooks::Hook;

#[derive(Default)]
pub(super) struct Declarations {
    pub imported: Option<crate::sema::crates::ImportedOwners>,
    pub schemas: HashMap<DefId, Schema>,
    pub members: HashMap<DefId, DefId>,
    pub inherent_members: HashMap<DefId, DefId>,
    callables: HashMap<DefId, (CallTrait, crate::resolve::unit::NodeKey)>,
    output: Option<DefId>,
    pub hooks: Vec<super::super::model::Function>,
    pub hook_members: Vec<(DefId, DefId)>,
    pub origins: HashMap<DefId, crate::resolve::unit::NodeKey>,
    callable_definitions: [Option<DefId>; 3],
    operations: HashMap<DefId, (Hook, DefId)>,
    pub(super) comparisons: Option<Box<super::comparisons::Comparisons>>,
}
impl Declarations {
    pub(super) fn accepts_imported(&self) -> bool {
        self.imported.is_none()
            && self.schemas.is_empty()
            && self.members.is_empty()
            && self.inherent_members.is_empty()
            && self.callables.is_empty()
            && self.output.is_none()
            && self.hooks.is_empty()
            && self.hook_members.is_empty()
            && self.origins.is_empty()
            && self.callable_definitions.iter().all(Option::is_none)
            && self.operations.is_empty()
            && self.comparisons.is_none()
    }
}
impl Inventory {
    /// Publish validated real source identities without synthetic members, schemas
    /// or function bodies. The ordinary inventory remains their sole owner.
    pub(in crate::sema) fn register_source_callables(
        &mut self,
        entries: &[(crate::resolve::bindings::Declaration, CallTrait, DefId)],
        output: Option<crate::resolve::bindings::Declaration>,
    ) -> Result<(), &'static str> {
        if entries.is_empty() {
            return Ok(());
        }
        if self
            .language
            .as_ref()
            .is_some_and(|l| !l.callables.is_empty() || l.output.is_some())
        {
            return Err("source callable publication cannot replace existing identities");
        }
        for &(declaration, _, method) in entries {
            let owner = self
                .owner(declaration.definition)
                .ok_or("missing source callable owner")?;
            let owner = self
                .get(owner)
                .ok_or("missing source callable declaration")?;
            if owner.source != Some(declaration.origin)
                || self.schema(declaration.definition).is_none()
                || self.trait_owner(method) != Some(declaration.definition)
                || self.language.as_ref().is_some_and(|l| {
                    l.schemas.contains_key(&declaration.definition)
                        || l.members.contains_key(&method)
                })
            {
                return Err("source callable publication requires ordinary source ownership");
            }
        }
        if let Some(output) = output
            && self.trait_owner(output.definition) != output.parent
        {
            return Err("source callable Output has a different owner");
        }
        for &(declaration, kind, _) in entries {
            self.declare_callable(declaration.definition, kind, None, declaration.origin);
        }
        self.language.get_or_insert_default().output = output.map(|d| d.definition);
        Ok(())
    }
    pub(in crate::sema) fn library_impl_member(
        &mut self,
        owner: DefId,
        member: DefId,
    ) -> Result<(), &'static str> {
        let language = self
            .language
            .as_mut()
            .ok_or("missing library member registry")?;
        if language.members.remove(&member) != Some(owner) {
            return Err("library implementation member has a different owner");
        }
        if language.inherent_members.insert(member, owner).is_some() {
            return Err("library implementation member already registered");
        }
        Ok(())
    }
    pub(in crate::sema) fn register_library_type_member(&mut self, owner: DefId, member: DefId) {
        let language = self.language.get_or_insert_default();
        if language.members.insert(member, owner).is_none() {
            language.hook_members.push((member, owner));
        }
    }
    pub(in crate::sema) fn declare_operation(
        &mut self,
        definition: DefId,
        kind: Hook,
        method: DefId,
    ) {
        self.language
            .get_or_insert_with(Default::default)
            .operations
            .insert(definition, (kind, method));
    }
    pub(in crate::sema) fn operation(&self, definition: DefId) -> Option<(Hook, DefId)> {
        self.language.as_ref()?.operations.get(&definition).copied()
    }
    pub(in crate::sema) fn declare_callable(
        &mut self,
        definition: DefId,
        kind: CallTrait,
        output: Option<DefId>,
        origin: crate::resolve::unit::NodeKey,
    ) {
        let language = self.language.get_or_insert_with(Default::default);
        language.callables.insert(definition, (kind, origin));
        language.callable_definitions[kind as usize] = Some(definition);
        if output.is_some() {
            language.output = output;
        }
    }

    pub(in crate::sema) fn callable(&self, definition: DefId) -> Option<CallTrait> {
        self.language
            .as_ref()?
            .callables
            .get(&definition)
            .map(|&(kind, _)| kind)
    }

    pub(in crate::sema) fn language_origin(
        &self,
        definition: DefId,
    ) -> Option<crate::resolve::unit::NodeKey> {
        let language = self.language.as_ref()?;
        language
            .callables
            .get(&definition)
            .map(|&(_, origin)| origin)
            .or_else(|| language.origins.get(&definition).copied())
    }

    pub(in crate::sema) fn callable_definition(&self, kind: CallTrait) -> Option<DefId> {
        self.language.as_ref()?.callable_definitions[kind as usize]
    }

    pub(in crate::sema) fn callable_output(&self) -> Option<DefId> {
        self.language.as_ref()?.output
    }

    pub(in crate::sema) fn declare_language_trait(
        &mut self,
        definition: DefId,
        schema: Schema,
        member: Option<DefId>,
    ) -> Result<(), &'static str> {
        let language = self.language.get_or_insert_with(Default::default);
        if language.schemas.contains_key(&definition)
            || member.is_some_and(|id| language.members.contains_key(&id))
        {
            return Err("language trait contract already declared");
        }
        language.schemas.insert(definition, schema);
        if let Some(member) = member {
            language.members.insert(member, definition);
        }
        Ok(())
    }
}

impl Inventory {
    pub(in crate::sema) fn register_language_members(
        &mut self,
        definition: DefId,
        origin: crate::resolve::unit::NodeKey,
        method: super::super::model::Function,
        output: Option<DefId>,
    ) {
        let language = self.language.get_or_insert_with(Default::default);
        language.origins.insert(definition, origin);
        language.members.insert(method.definition, definition);
        language.hook_members.push((method.definition, definition));
        language.hooks.push(method);
        if let Some(output) = output {
            language.members.insert(output, definition);
            language.hook_members.push((output, definition));
        }
    }
    pub(crate) fn language_functions(
        &self,
    ) -> impl Iterator<Item = &super::super::model::Function> {
        self.language
            .iter()
            .flat_map(|language| language.hooks.iter())
    }
    pub(crate) fn language_function(&self, index: usize) -> Option<&super::super::model::Function> {
        self.language.as_ref()?.hooks.get(index)
    }
    pub(in crate::sema) fn member_owner(&self, definition: DefId) -> Option<DefId> {
        self.language
            .as_ref()
            .and_then(|language| {
                language
                    .members
                    .get(&definition)
                    .or_else(|| language.inherent_members.get(&definition))
                    .copied()
            })
            .or_else(|| {
                self.get(self.owner(definition)?)
                    .map(|owner| owner.self_definition)
            })
    }
    pub(in crate::sema) fn declaration_members(&self) -> impl Iterator<Item = (DefId, DefId)> + '_ {
        self.members
            .iter()
            .map(|member| {
                (
                    member.definition,
                    self.owners[member.owner.0 as usize].self_definition,
                )
            })
            .chain(
                self.language
                    .iter()
                    .flat_map(|language| language.hook_members.iter().copied()),
            )
    }
    pub(in crate::sema) fn declaration_owner(&self, definition: DefId) -> Option<DefId> {
        if self
            .language
            .as_ref()
            .is_some_and(|language| language.schemas.contains_key(&definition))
        {
            Some(definition)
        } else {
            self.get(self.owner(definition)?)
                .map(|owner| owner.self_definition)
        }
    }
}

impl Inventory {
    pub(in crate::sema) fn declare_language_inherent(
        &mut self,
        owner: DefId,
        schema: Schema,
        method: DefId,
        origin: crate::resolve::unit::NodeKey,
    ) -> Result<(), &'static str> {
        let language = self.language.get_or_insert_with(Default::default);
        if language.inherent_members.insert(method, owner).is_some() {
            return Err("language inherent method already declared");
        }
        language.schemas.entry(owner).or_insert(schema);
        language.origins.insert(owner, origin);
        language.hook_members.push((method, owner));
        Ok(())
    }
}
