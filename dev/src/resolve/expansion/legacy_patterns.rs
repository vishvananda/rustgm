//! Map a checked matcher migration site through its actual source unit.
use super::{Error, NodeKey, Report};
use crate::expand::macros::{legacy_or_patterns, model::Definition};

impl Report {
    pub(crate) fn legacy_pattern_sites(
        &self,
        key: NodeKey,
        definition: &Definition,
    ) -> Result<Vec<u32>, Error> {
        let unit = self.package.unit(key.file);
        legacy_or_patterns(definition, &self.package.names, unit.source.text())
            .map_err(|error| Error::at(key.file, error.offset, error.message))
    }
}
