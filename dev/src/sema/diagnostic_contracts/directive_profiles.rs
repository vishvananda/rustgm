//! Capped bring-up defers diagnostic rendering, preserving lexical/hard errors.
use crate::{
    lex::token::Punctuation,
    resolve::{expansion::Report, package::Error, unit::FileId},
    sema::model::Declarations,
    syntax::{
        attribute::{Attribute, Input},
        token_tree::Forest,
    },
};

/// Crate requests only control diagnostic metadata; no operation is authorized.
#[derive(Default)]
pub(super) struct Requests([Option<bool>; 2]);
impl Requests {
    fn enabled(&mut self, report: &Report, unmatched: bool) -> Result<bool, Error> {
        let index = usize::from(unmatched);
        if let Some(enabled) = self.0[index] {
            return Ok(enabled);
        }
        let enabled = report
            .package
            .requested_diagnostic_directive(unmatched)
            .map_err(Error::unlocated)?;
        self.0[index] = Some(enabled);
        Ok(enabled)
    }
}

fn recognized(attribute: Attribute, forest: &Forest, source: &str) -> bool {
    let path = attribute.path;
    path.end == path.start + 3
        && forest.name(source, path.start) == Some("diagnostic")
        && forest.punctuation(path.start + 1, Punctuation::ColonColon)
        && matches!(
            forest.name(source, path.start + 2),
            Some("on_const" | "on_unmatched_args")
        )
}

pub(super) fn validate_literals(
    file: FileId,
    source: &str,
    attribute: Attribute,
    forest: &Forest,
) -> Result<(), Error> {
    let range = match attribute.input {
        Input::Delimited(group) => forest.children(group),
        Input::Equals(range) => Some(range),
        Input::Empty | Input::Doc(_) => None,
    };
    if let Some(range) = range {
        // Flat retained input visits each actual token, including nested groups,
        // once. Disabled owners still require valid literal escapes.
        for index in range.start..range.end {
            if let Some(token) = forest.token(index) {
                crate::literal::validate_attribute_text(source, token)
                    .map_err(|error| Error::at(file, error.offset, error.message))?;
            }
        }
    }
    Ok(())
}

pub(super) fn defer(
    report: &Report,
    facts: &Declarations,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
    requests: &mut Requests,
) -> Result<bool, Error> {
    let source = report.package.units[file.0 as usize].source.text();
    if !recognized(attribute, forest, source) {
        return Ok(false);
    }
    if attribute.unsafe_wrapper {
        return Err(Error::at(
            file,
            attribute.span.start,
            "diagnostic directive cannot use an unsafe attribute wrapper",
        ));
    }
    validate_literals(file, source, attribute, forest)?;
    facts.require_lint_context(report)?;
    if !report.package.lint_cap().suppresses_lints() {
        let unmatched = forest.name(source, attribute.path.start + 2) == Some("on_unmatched_args");
        // The pinned handlers ignore metadata when their crate feature is absent;
        // allow_internal_unstable does not enable either diagnostic handler.
        if !requests.enabled(report, unmatched)? {
            return Ok(true);
        }
        return Err(Error::at(
            file,
            attribute.span.start,
            "uncapped const/macro diagnostic directive profile is not implemented",
        ));
    }
    // No diagnostic table, feature capability, obligation or body permission.
    Ok(true)
}

#[cfg(test)]
#[path = "../../../tests/invariants/diagnostic_directives.rs"]
mod tests;
