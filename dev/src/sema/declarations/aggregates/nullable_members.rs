//! Source nullable shapes publish physical data before layout demand.
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
        if !self.report.package.uses_source_bindings() {
            return Ok(());
        }
        let option = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(LangItem::Option))
            .filter(|declaration| declaration.definition.0 == item.0);
        let entry = &self.nominals.entries[self.nominals.index[&item]];
        let key = entry.key;
        if option.is_some_and(|declaration| declaration.origin != key) {
            return Err(self.error(key, "source Option owner changed"));
        }
        // Explicit representations keep their actual ordinary layout. The
        // historical course profile is selected independently by its driver.
        if entry.kind != crate::types::aggregate::Kind::Enum
            || entry.repr.representation != Representation::Rust
            || entry.repr.tag.is_some()
            || entry.repr.align.is_some()
            || entry.repr.packed.is_some()
        {
            return Ok(());
        }
        if entry.groups.len() != 2 || entry.groups.iter().any(|g| g.discriminant.is_some()) {
            return if option.is_some() {
                Err(self.error(
                    key,
                    "source Option physical profile requires two implicit variants",
                ))
            } else {
                Ok(())
            };
        }
        let storage_limit = self.reserve_nullable_policy()?;
        let (empty, payload) = if option.is_some() {
            let parameter = self.option_formal(item, key)?;
            (
                self.option_variant(item, key, LangItem::OptionNone, None)?,
                self.option_variant(item, key, LangItem::OptionSome, Some(parameter))?,
            )
        } else {
            let Some(variants) = self.nullable_shape_variants(item, key)? else {
                return Ok(());
            };
            variants
        };
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
            .map_err(|error| self.error(key, error))
    }
    /// Actual empty/single-field storage needs no Option identity to use a niche.
    /// This grants no variant, trait or API permission.
    fn nullable_shape_variants(
        &self,
        item: ItemId,
        key: NodeKey,
    ) -> Result<Option<(u32, u32)>, Error> {
        let shape = self.facts.aggregates.get(item).ok_or_else(|| {
            self.error(
                key,
                "source nullable enum lacks its completed storage shape",
            )
        })?;
        let [first, second] = self.facts.aggregates.variants(shape) else {
            return Err(self.error(key, "source nullable enum lost its two field groups"));
        };
        let arity = |variant: &crate::types::aggregate::Variant| {
            self.facts
                .types
                .list(variant.fields)
                .map(<[TypeId]>::len)
                .ok_or_else(|| self.error(key, "source nullable enum lost its actual fields"))
        };
        Ok(match (arity(first)?, arity(second)?) {
            (0, 1) => Some((0, 1)),
            (1, 0) => Some((1, 0)),
            _ => None,
        })
    }
    fn reserve_nullable_policy(&mut self) -> Result<usize, Error> {
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
        Ok(storage_limit)
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
