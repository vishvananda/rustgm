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
        self.language
            .as_ref()?
            .callables
            .get(&definition)
            .map(|&(_, origin)| origin)
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
