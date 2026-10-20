//! Typed language contracts have no source owner or generated AST declaration.
use super::{DefId, Inventory, Schema};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Declarations {
    pub schemas: HashMap<DefId, Schema>,
    pub members: HashMap<DefId, DefId>,
}
impl Inventory {
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
