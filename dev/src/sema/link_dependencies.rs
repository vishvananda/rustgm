//! Validate native metadata independently of foreign declaration visibility/safety.
use crate::{
    lex::token::Punctuation,
    resolve::{expansion::Report, package::Error, unit::NodeKey},
    sema::model::Declarations,
    support::linker_libraries::{Kind, Library},
    syntax::{
        ast::Kind as Syntax, attribute::Attribute, item::ItemKind, link_arguments,
        token_tree::Forest,
    },
};

#[derive(Default)]
pub(super) struct Draft {
    enabled: Option<bool>,
}
impl Draft {
    pub(super) fn consume(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        key: NodeKey,
        attribute: Attribute,
        effect: usize,
        forest: &Forest,
    ) -> Result<(), Error> {
        let error = |message| Error::at(key.file, attribute.span.start, message);
        let unit = report.package.unit(key.file);
        let condition = unit.native_link_condition(effect);
        if condition.is_some() {
            let enabled = if let Some(enabled) = self.enabled {
                enabled
            } else {
                let enabled = report.package.requested_link_cfg().map_err(error)?;
                self.enabled = Some(enabled);
                enabled
            };
            if !enabled {
                return Err(error(
                    "conditional native links require the link_cfg crate feature",
                ));
            }
        }
        let library = parse(report, key, attribute, forest)?;
        crate::sema::header_attributes::reserve_source_contracts(facts, 1, library.name().len())?;
        let foreign = matches!(unit.configured.ast.node(key.node).kind,
            Syntax::Item(index) if matches!(unit.configured.ast.items[index as usize].kind, ItemKind::ExternBlock { .. }));
        if !foreign {
            return facts.must_use_diagnostic(
                report,
                key,
                attribute.span.start,
                0,
                crate::support::diagnostics::WarningCode::UnusedAttributes,
                "link attribute requires an extern block".into(),
            );
        }
        if condition != Some(false) {
            facts
                .foreign
                .get_or_insert_with(Box::default)
                .add_native_library(library)
                .map_err(error)?;
        }
        Ok(())
    }
}

fn parse(
    report: &Report,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Library, Error> {
    let unit = report.package.unit(key.file);
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let (mut name, mut kind) = (None, None);
    for argument in link_arguments::arguments(forest, attribute)
        .map_err(|failure| Error::at(key.file, failure.offset, failure.message))?
    {
        let argument_name = forest.name(unit.source.text(), argument.start);
        if argument_name == Some("cfg") {
            continue;
        }
        if argument.start + 3 != argument.end
            || !forest.punctuation(argument.start + 1, Punctuation::Equal)
        {
            return Err(error("link metadata requires name=value string arguments"));
        }
        let value =
            crate::syntax::configuration::string(forest, unit.source.text(), argument.start + 2)
                .map_err(|failure| Error::at(key.file, failure.offset, failure.message))?;
        match argument_name {
            Some("name") if name.is_none() => name = Some(value),
            Some("kind") if kind.is_none() => {
                kind = Some(match value.as_ref() {
                    "dylib" => Kind::Dylib,
                    "static" => Kind::Static,
                    "framework" => {
                        return Err(error("framework native libraries require an Apple target"));
                    }
                    _ => return Err(error("unsupported native library kind")),
                });
            }
            Some("modifiers" | "import_name_type") => {
                return Err(error(
                    "native link modifiers/import names are not implemented",
                ));
            }
            _ => return Err(error("unknown or duplicate native link argument")),
        }
    }
    Library::new(
        name.as_deref()
            .ok_or_else(|| error("native link requires a library name"))?,
        kind.unwrap_or(Kind::Dylib),
    )
    .map_err(error)
}

#[cfg(test)]
#[path = "../../tests/invariants/link_requirements.rs"]
mod tests;
