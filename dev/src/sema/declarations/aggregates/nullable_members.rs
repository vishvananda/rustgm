//! The actual source Option header publishes physical data before layout demand.
use super::{Checker, DefId, Error, ItemId, NodeKey, Primitive, Type, TypeId};
use crate::{
    support::bindings::LangItem,
    types::{
        aggregate::{NullableEnum, Representation},
        arguments::Argument,
        parameters::ParameterKind,
    },
};

impl Checker<'_> {
    pub(super) fn source_nullable_members(&mut self, item: ItemId) -> Result<(), Error> {
        let Some(declaration) = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(LangItem::Option))
        else {
            return Ok(());
        };
        if declaration.definition.0 != item.0 {
            return Ok(());
        }
        let entry = &self.nominals.entries[self.nominals.index[&item]];
        let key = entry.key;
        if key != declaration.origin {
            return Err(self.error(key, "source Option owner changed"));
        }
        // Explicit source representations keep their actual ordinary layout.
        if entry.repr.representation != Representation::Rust
            || entry.repr.tag.is_some()
            || entry.repr.align.is_some()
            || entry.repr.packed.is_some()
        {
            return Ok(());
        }
        let growth = self.facts.aggregates.nullable_publication_growth();
        let storage_limit = self
            .facts
            .aggregates
            .nullable_storage_bytes()
            .saturating_add(growth.bytes);
        crate::sema::header_attributes::reserve_source_contracts(
            &mut self.facts,
            16_usize.saturating_add(growth.work),
            growth.bytes,
        )?;
        let entry = &self.nominals.entries[self.nominals.index[&item]];
        if entry.groups.len() != 2 || entry.groups.iter().any(|g| g.discriminant.is_some()) {
            return Err(self.error(
                key,
                "source Option physical profile requires two implicit variants",
            ));
        }
        let parameter = self.option_formal(item, key)?;
        let empty = self.option_variant(item, key, LangItem::OptionNone, None)?;
        let payload = self.option_variant(item, key, LangItem::OptionSome, Some(parameter))?;
        self.facts
            .aggregates
            .register_nullable_enum(
                &self.facts.types,
                item,
                NullableEnum {
                    empty,
                    payload,
                    field: 0,
                    fallback_tag: Some(Primitive::U8),
                },
                Some(storage_limit),
            )
            .map_err(|e| self.error(key, e))
    }
    fn option_formal(&self, item: ItemId, key: NodeKey) -> Result<TypeId, Error> {
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&DefId(item.0))
            .ok_or_else(|| self.error(key, "source Option requires its own type formal"))?;
        let [parameter] = parameters.as_slice() else {
            return Err(self.error(key, "source Option requires exactly one own type formal"));
        };
        let Argument::Type(ty) = parameter.argument else {
            return Err(self.error(key, "source Option requires a type formal"));
        };
        let Some(Type::Parameter(id)) = self.facts.types.kind(ty) else {
            return Err(self.error(key, "source Option formal identity changed"));
        };
        if self.facts.types.parameter(id).is_none_or(|p| {
            p.owner != item
                || p.index != 0
                || !matches!(p.kind, ParameterKind::Type { sized: true })
        }) {
            return Err(self.error(key, "source Option requires its own Sized type formal"));
        }
        Ok(ty)
    }
    fn option_variant(
        &self,
        item: ItemId,
        key: NodeKey,
        role: LangItem,
        field: Option<TypeId>,
    ) -> Result<u32, Error> {
        let declaration = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(role))
            .ok_or_else(|| {
                self.error(
                    key,
                    "source Option physical profile requires actual Some and None",
                )
            })?;
        let variants = self.facts.members.variants(item);
        let id = variants
            .iter()
            .copied()
            .find(|id| {
                self.facts.members.variants[id.0 as usize].origin.source()
                    == Some(declaration.origin)
            })
            .ok_or_else(|| {
                self.error(key, "source Option variant belongs to another source owner")
            })?;
        let variant = &self.facts.members.variants[id.0 as usize];
        let fields = self
            .facts
            .members
            .fields(id)
            .ok_or_else(|| self.error(key, "source Option fields missing"))?;
        if variant.owner != item
            || variants.get(variant.ordinal as usize) != Some(&id)
            || fields.len() != usize::from(field.is_some())
            || fields.first().map(|f| f.ty) != field
        {
            return Err(self.error(
                key,
                "source Option field differs from its actual type formal",
            ));
        }
        let group =
            &self.nominals.entries[self.nominals.index[&item]].groups[variant.ordinal as usize];
        if group.key != declaration.origin
            || (field.is_some() && group.shape != crate::syntax::item::Shape::Tuple)
        {
            return Err(self.error(key, "source Option variant shape or origin changed"));
        }
        Ok(variant.ordinal)
    }
}
