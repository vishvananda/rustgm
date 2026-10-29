//! Demand the caller site's actual nominal data without unrelated bodies or `PanicInfo`.
use super::{Checker, DefId, Error, ItemId, Work};
use crate::{sema::compiler_items, support::bindings::LangItem};

impl Checker<'_> {
    pub(super) fn caller_location_data(&mut self, owner: DefId, finish: bool) -> Result<(), Error> {
        let catalog = self.facts.source_bindings.as_deref().ok_or_else(|| {
            Error::unlocated("caller-location dependency lost its source catalog")
        })?;
        let location = catalog
            .language(LangItem::PanicLocation)
            .ok_or_else(|| Error::unlocated("caller-location dependency lost actual Location"))?;
        if location.definition != owner {
            return Err(self.error(
                location.origin,
                "caller-location dependency changed its owner",
            ));
        }
        if finish {
            compiler_items::phantom::complete(self.report, &mut self.facts)?;
            return compiler_items::site_records::complete_location(self.report, &mut self.facts);
        }
        let phantom = catalog.language(LangItem::PhantomData).ok_or_else(|| {
            self.error(
                location.origin,
                "caller-location data requires actual PhantomData",
            )
        })?;
        let nonnull = self
            .report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(crate::resolve::bindings::Reservations::nonnull_declaration)
            .ok_or_else(|| {
                self.error(
                    location.origin,
                    "caller-location data requires actual NonNull",
                )
            })?;
        self.work.push(Work::CallerLocation(owner, true));
        for definition in [owner, nonnull.definition, phantom.definition] {
            self.work.push(Work::Fields(ItemId(definition.0), false));
        }
        for definition in [owner, nonnull.definition, phantom.definition] {
            self.work.push(Work::Definition(definition, false));
        }
        Ok(())
    }
}
