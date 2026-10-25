//! Closed source publication checks retained must-use and inline contracts.
use super::{Declarations, Report, Source, s};
use crate::{
    resolve::{
        model::{DefId, DefKind, Namespace},
        package::Error,
        unit::NodeKey,
    },
    syntax::{ast::Kind, item::ItemKind},
};

impl Source {
    pub(super) fn require_plain_attributes(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        self.require_attribute_context(report, facts)?;
        for index in 0..self.stage.definitions.len() {
            self.stage.charge(1).map_err(Error::unlocated)?;
            let Some((definition, declaration)) =
                selected_header(&self.stage, report, facts, index)?
            else {
                continue;
            };
            require_defaultness_transport(report, declaration)?;
            facts.reject_specialization_transport(report, definition)?;
            self.stage.charge(2).map_err(Error::unlocated)?;
            if let Some(contract) = facts.must_use_contract(report, definition)? {
                let source = contract.origin.source().ok_or_else(|| {
                    Error::unlocated("source publication borrowed an imported contract")
                })?;
                if source.declaration != declaration || source.owner.file != declaration.file {
                    return Err(Error::at(
                        source.owner.file,
                        source.span.start,
                        "source must_use contract names another actual declaration",
                    ));
                }
                return Err(report.diagnostic(Error::at(
                    source.owner.file,
                    source.span.start,
                    "source must_use attribute contract transport is not implemented",
                )));
            }
            if let Some(hint) = facts.inline_hint_for_definition(report, definition)? {
                let source = hint.origin.source().ok_or_else(|| {
                    Error::unlocated("source publication borrowed an imported hint")
                })?;
                if source.declaration != declaration || source.owner.file != declaration.file {
                    return Err(Error::at(
                        source.owner.file,
                        source.span.start,
                        "source inline hint names another actual declaration",
                    ));
                }
                return Err(report.diagnostic(Error::at(
                    source.owner.file,
                    source.span.start,
                    "source inline attribute contract transport is not implemented",
                )));
            }
        }
        Ok(())
    }
    pub(super) fn require_attribute_context(
        &self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        let provider = report.package.source_core().ok_or_else(|| {
            Error::unlocated("source attribute publication requires its actual own-provider report")
        })?;
        if provider.current != provider.provider {
            return Err(Error::unlocated(
                "source attribute publication belongs to another provider",
            ));
        }
        facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| {
                Error::unlocated("source attribute publication lacks its actual provider catalog")
            })?
            .require_provider_report(report)
            .map_err(Error::unlocated)?;
        if self.stage.definitions.len() != report.resolution.definitions.len() {
            return Err(Error::unlocated(
                "source attribute stage belongs to another declaration arena",
            ));
        }
        Ok(())
    }
}

pub(super) fn selected_header(
    stage: &crate::sema::crates::authored_export::Stage,
    report: &Report,
    facts: &Declarations,
    index: usize,
) -> Result<Option<(DefId, NodeKey)>, Error> {
    let Some(row) = stage.definitions[index] else {
        return Ok(None);
    };
    let actual = &report.resolution.definitions[index];
    let expected = match actual.kind {
        DefKind::Function => s::DefinitionKind::Function,
        DefKind::Trait => s::DefinitionKind::Trait,
        _ => return Ok(None),
    };
    let definition = DefId(
        u32::try_from(index)
            .map_err(|_| Error::unlocated("source attribute definition index overflow"))?,
    );
    let declaration = actual.node.ok_or_else(|| {
        Error::unlocated("source attribute publication cannot infer imported contract absence")
    })?;
    let staged = stage
        .rows
        .declarations
        .get(row.0 as usize)
        .ok_or_else(|| Error::unlocated("source attribute declaration row is missing"))?;
    let namespace = if actual.kind == DefKind::Function {
        Namespace::Value
    } else {
        Namespace::Type
    };
    if staged.definition.kind != expected
        || staged.name.as_deref() != Some(report.package.names.get(actual.name))
        || actual.namespace != namespace
    {
        return Err(Error::at(
            declaration.file,
            0,
            "source attribute header differs from its staged declaration",
        ));
    }
    source_header(report, facts, definition, declaration, actual.kind)?;
    Ok(Some((definition, declaration)))
}

fn source_header(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
    declaration: NodeKey,
    kind: DefKind,
) -> Result<(), Error> {
    let unit = report
        .package
        .units
        .get(declaration.file.0 as usize)
        .ok_or_else(|| Error::unlocated("source attribute header unit is absent"))?;
    let node = unit
        .configured
        .ast
        .nodes
        .get(declaration.node.get() as usize)
        .ok_or_else(|| Error::unlocated("source attribute header syntax is absent"))?;
    let valid = match node.kind {
        Kind::Item(item) => unit
            .configured
            .ast
            .items
            .get(item as usize)
            .is_some_and(|item| {
                matches!(
                    (kind, item.kind),
                    (DefKind::Function, ItemKind::Function { .. })
                        | (DefKind::Trait, ItemKind::Trait { .. })
                )
            }),
        _ => false,
    };
    if !valid || unit.include_context.is_some() {
        return Err(Error::at(
            declaration.file,
            node.span.start,
            "source attribute header lost its actual local declaration",
        ));
    }
    if kind == DefKind::Function {
        let header = facts
            .function_header(definition)
            .map_err(Error::unlocated)?
            .ok_or_else(|| Error::unlocated("source attribute function has no semantic header"))?;
        if header.declaration.source() != Some(declaration) {
            return Err(Error::at(
                declaration.file,
                node.span.start,
                "source attribute function belongs to another actual declaration",
            ));
        }
    }
    Ok(())
}

/// Defaultness is a semantic implementation contract, absent from both v0/v1.
pub(super) fn require_defaultness_transport(
    report: &Report,
    declaration: NodeKey,
) -> Result<(), Error> {
    let Some(defaultness) = report.active_defaultness(declaration)? else {
        return Ok(());
    };
    if defaultness.node != declaration.node {
        return Err(Error::at(
            declaration.file,
            defaultness.keyword.start,
            "source defaultness header names another declaration",
        ));
    }
    Err(report.diagnostic(Error::at(
        declaration.file,
        defaultness.keyword.start,
        "source defaultness contract transport is not implemented",
    )))
}
