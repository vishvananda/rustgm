//! Template feature authority is checked before installing a compiled definition.
use super::{Engine, Error, FileId, Sources};
use crate::expand::macros::model::{Definition, Kind};
#[cfg(test)]
#[path = "../../../tests/invariants/metavariable_concat.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../tests/invariants/metavariable_expressions.rs"]
mod expression_tests;

impl<S: Sources> Engine<'_, S> {
    pub(super) fn check_template_features(
        &self,
        file: FileId,
        grammar: &Definition,
    ) -> Result<(), Error> {
        let package = &self.loader.package;
        let concat = package
            .requested_metavariable_concat()
            .map_err(Error::unlocated)?;
        let expressions = package
            .requested_metavariable_expressions()
            .map_err(Error::unlocated)?;
        if concat && expressions {
            return Ok(());
        }
        if let Some(context) = package.macro_context().map_err(Error::unlocated)? {
            context
                .contracts()
                .charge(grammar.nodes.len(), 0)
                .map_err(|message| Error::at(file, 0, message))?;
        }
        for node in &grammar.nodes {
            let missing = match node.kind {
                Kind::Concat(_) if !concat => {
                    Some("metavariable concat requires the crate feature macro_metavar_expr_concat")
                }
                Kind::Expression(_) | Kind::EscapedDollar(_) if !expressions => {
                    Some("metavariable expression requires the crate feature macro_metavar_expr")
                }
                _ => None,
            };
            if let Some(message) = missing {
                return Err(Error::at(file, node.span.start, message));
            }
        }
        Ok(())
    }
}
