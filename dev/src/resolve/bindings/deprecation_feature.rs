//! Optional replacement metadata requires its real source feature request.
use super::policy::Feature;
use crate::resolve::package::Package;
impl Package {
    pub(crate) fn requested_deprecation_suggestion(&self) -> Result<bool, &'static str> {
        if let Some(source) = self.staged_consumer() {
            source.require_package(self)?;
            return Ok(source.features.has(Feature::DeprecatedSuggestion));
        }
        if let Some(client) = self.source_client() {
            if client.current().name() != self.crate_name || client.current() == client.provider() {
                return Err("deprecation feature client identity changed");
            }
            client.validate_inventory()?;
            return Ok(client.features.has(Feature::DeprecatedSuggestion));
        }
        Ok(self
            .macro_context()?
            .is_some_and(|context| context.requested(Feature::DeprecatedSuggestion)))
    }
}
