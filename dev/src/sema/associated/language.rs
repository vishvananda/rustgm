//! Typed language contracts have no source owner or generated AST declaration.
use super::{DefId, Inventory, Schema};
use crate::support::call_traits::CallTrait;
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Declarations {
    pub schemas: HashMap<DefId, Schema>,
    pub members: HashMap<DefId, DefId>,
    callables: HashMap<DefId, (CallTrait, crate::resolve::unit::NodeKey)>,
    output: Option<DefId>,
    pub hooks: Vec<super::super::model::Function>,
    pub hook_members: Vec<(DefId, DefId)>,
    pub origins: HashMap<DefId, crate::resolve::unit::NodeKey>,
    callable_definitions: [Option<DefId>; 3],
}
impl Inventory {
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
        output: Option<(DefId, crate::resolve::names::NameId)>,
    ) {
        let language = self.language.get_or_insert_with(Default::default);
        language.origins.insert(definition, origin);
        language.members.insert(method.definition, definition);
        language.hook_members.push((method.definition, definition));
        language.hooks.push(method);
        if let Some((output, name)) = output {
            language.members.insert(output, definition);
            language.hook_members.push((output, definition));
            self.type_names.insert(output, name);
        }
    }
    pub(in crate::sema) fn language_functions(
        &self,
    ) -> impl Iterator<Item = &super::super::model::Function> {
        self.language
            .iter()
            .flat_map(|language| language.hooks.iter())
    }
    pub(in crate::sema) fn member_owner(&self, definition: DefId) -> Option<DefId> {
        self.language
            .as_ref()
            .and_then(|language| language.members.get(&definition).copied())
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
