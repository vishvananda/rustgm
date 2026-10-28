//! Retain replacement metadata without weakening the language feature gate.
use super::producer::Producer;
use crate::resolve::{expansion::Report, package::Error};
#[cfg(test)]
#[path = "../../../tests/invariants/deprecation_suggestions.rs"]
mod tests;
impl Producer {
    pub(crate) fn validate_suggestions(&self, report: &Report) -> Result<(), Error> {
        if !self.has_suggestions
            || report
                .package
                .requested_deprecation_suggestion()
                .map_err(Error::unlocated)?
        {
            return Ok(());
        }
        for owner in &self.owners {
            self.attribute_read(owner.key, 1)?;
            let Some((replacement, span)) = owner
                .parsed
                .deprecated
                .as_ref()
                .and_then(|deprecation| deprecation.suggestion.as_ref())
            else {
                continue;
            };
            self.attribute_read(owner.key, replacement.len().saturating_add(1))?;
            return Err(Error::at(
                owner.key.file,
                span.start,
                "deprecation suggestion requires deprecated_suggestion",
            ));
        }
        Ok(())
    }
}
