//! Template feature authority is checked before installing a compiled definition.
use super::{Engine, Error, FileId, Sources};
use crate::expand::macros::model::{Definition, Kind};
#[cfg(test)]
#[path = "../../../tests/invariants/metavariable_concat.rs"]
mod tests;

impl<S: Sources> Engine<'_, S> {
    pub(super) fn check_concat(&self, file: FileId, grammar: &Definition) -> Result<(), Error> {
        let package = &self.loader.package;
        if package
            .requested_metavariable_concat()
            .map_err(Error::unlocated)?
        {
            return Ok(());
        }
        if let Some(context) = package.macro_context().map_err(Error::unlocated)? {
            context
                .contracts()
                .charge(grammar.nodes.len(), 0)
                .map_err(|message| Error::at(file, 0, message))?;
        }
        for node in &grammar.nodes {
            if matches!(node.kind, Kind::Concat(_)) {
                return Err(Error::at(
                    file,
                    node.span.start,
                    "metavariable concat requires the crate feature macro_metavar_expr_concat",
                ));
            }
        }
        Ok(())
    }
}
