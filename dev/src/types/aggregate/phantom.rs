//! A checked marker owns its real type argument without adding physical storage.
use super::{Definitions, Kind};
use crate::types::{GenericArgs, ItemId, Store, TypeId, arguments::Argument};

impl Definitions {
    /// Logical ownership of a checked source marker, never a callable/trait proof.
    /// # Errors
    /// Rejects changed marker shape or incomplete/malformed actual arguments.
    pub fn phantom_argument(
        &self,
        types: &Store,
        ty: TypeId,
    ) -> Result<Option<TypeId>, &'static str> {
        let Some(crate::types::Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Ok(None);
        };
        if self.get(item).is_none_or(|d| d.kind != Kind::PhantomData) {
            return Ok(None);
        }
        self.phantom_argument_from(types, item, arguments).map(Some)
    }
    pub(crate) fn phantom_argument_from(
        &self,
        types: &Store,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<TypeId, &'static str> {
        let definition = self.get(item).ok_or("missing phantom declaration")?;
        if definition.kind != Kind::PhantomData {
            return Err("logical phantom argument requires its checked nominal category");
        }
        let [variant] = self.variants(definition) else {
            return Err("phantom declaration changed its actual field group");
        };
        if types
            .list(variant.fields)
            .is_none_or(|fields| !fields.is_empty())
        {
            return Err("phantom declaration cannot own physical fields");
        }
        let mut arguments = types
            .arguments(arguments)
            .ok_or("missing phantom arguments")?;
        let Some(Argument::Type(target)) = arguments.next() else {
            return Err("phantom declaration requires one actual type argument");
        };
        if arguments.next().is_some() || types.kind(target).is_none() {
            return Err("phantom declaration has invalid actual type arguments");
        }
        Ok(target)
    }
}
