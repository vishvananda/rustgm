//! Only canonical Option publication installs the nullable layout policy.
use super::{Error, ItemId, LibraryField, LibraryNominal, LibraryType};
use crate::sema::declarations::comparisons::LibraryContracts;
use crate::types::aggregate::NullableEnum;
impl LibraryContracts<'_> {
    pub(super) fn register_nullable_library_type(
        &mut self,
        declaration: &LibraryNominal,
        item: ItemId,
    ) -> Result<(), Error> {
        if declaration.kind != LibraryType::Option {
            return Ok(());
        }
        let mut empty = None;
        let mut payload = None;
        for (ordinal, variant) in declaration.variants.iter().enumerate() {
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| self.error(declaration.origin, "nullable variant ordinal overflow"))?;
            let slot = match variant.field {
                None => &mut empty,
                Some(LibraryField::Parameter(0)) => &mut payload,
                _ => {
                    return Err(self.error(
                        declaration.origin,
                        "canonical Option has a different payload descriptor",
                    ));
                }
            };
            if slot.replace(ordinal).is_some() {
                return Err(self.error(declaration.origin, "ambiguous canonical Option descriptor"));
            }
        }
        let policy = NullableEnum {
            empty: empty.ok_or_else(|| {
                self.error(declaration.origin, "canonical Option lacks empty variant")
            })?,
            payload: payload.ok_or_else(|| {
                self.error(declaration.origin, "canonical Option lacks payload variant")
            })?,
            field: 0,
        };
        self.facts
            .aggregates
            .register_nullable_enum(&self.facts.types, item, policy)
            .map_err(|error| self.error(declaration.origin, error))
    }
}
