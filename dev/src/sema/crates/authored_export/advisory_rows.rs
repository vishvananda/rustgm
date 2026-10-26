//! One actual selected-header collector owns source advisory completion.
use super::{Declarations, Report, Stage, collected_provider::CollectedScope, s};
use crate::{
    resolve::{
        model::{DefId, DefKind, Namespace},
        package::Error,
        unit::NodeKey,
    },
    sema::header_attributes::{ContractKnowledge, HeaderAttributeView},
    syntax::{ast::Kind, item::ItemKind},
};

#[derive(Clone, Copy)]
enum Collection {
    Headers,
    FormalScopes,
}

impl Stage {
    pub(crate) fn collect_header_columns(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.collect_header_columns_checked(report, facts, |_, _| Ok(()))
    }
    pub(crate) fn collect_header_columns_checked(
        &mut self,
        report: &Report,
        facts: &Declarations,
        mut check: impl FnMut(&mut Stage, NodeKey) -> Result<(), Error>,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.collect_columns(report, facts, Collection::Headers, |stage, _, source| {
            check(stage, source)
        })
    }
    pub(super) fn collect_formal_columns(
        &mut self,
        report: &Report,
        facts: &Declarations,
        check: impl FnMut(&mut Stage, DefId, NodeKey) -> Result<(), Error>,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.collect_columns(report, facts, Collection::FormalScopes, check)
    }
    fn collect_columns(
        &mut self,
        report: &Report,
        facts: &Declarations,
        mode: Collection,
        mut check: impl FnMut(&mut Stage, DefId, NodeKey) -> Result<(), Error>,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.require_attribute_context(report, facts)?;
        crate::sema::compiler_items::pointee::require_transport(report, facts)?;
        let count = self.rows.declarations.len();
        self.reserve(
            count.saturating_mul(size_of::<Option<s::HeaderContract>>())
                + size_of::<s::HeaderAttributeColumns>(),
        )
        .map_err(Error::unlocated)?;
        let mut completed: Vec<Option<s::HeaderContract>> = (0..count).map(|_| None).collect();
        for index in 0..self.definitions.len() {
            self.charge(1).map_err(Error::unlocated)?;
            let selected = match mode {
                Collection::Headers => self
                    .selected_header(report, facts, index)?
                    .map(|(definition, source)| CollectedScope::header_source(definition, source)),
                Collection::FormalScopes => self.collected_scope(report, facts, index)?,
            };
            let Some(selected) = selected else {
                continue;
            };
            let definition = selected.definition();
            let declaration = selected.source();
            check(self, definition, declaration)?;
            if !selected.header() {
                continue;
            }
            require_defaultness_transport(report, declaration)?;
            let row = self.definitions[index]
                .ok_or_else(|| Error::unlocated("selected attribute declaration disappeared"))?;
            let view = facts.publication_header_contracts(report, definition)?;
            if view
                .origin
                .source()
                .is_none_or(|source| source.declaration != declaration)
            {
                return Err(Error::at(
                    declaration.file,
                    0,
                    "source contract completion belongs to another actual header",
                ));
            }
            let contract = self.header_contract(row, view)?;
            if completed[row.0 as usize].replace(contract).is_some() {
                return Err(Error::at(
                    declaration.file,
                    0,
                    "source header contract emitted twice",
                ));
            }
        }
        self.finish_columns(completed)
    }
    fn finish_columns(
        &mut self,
        mut completed: Vec<Option<s::HeaderContract>>,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.reserve(
            self.rows
                .functions
                .len()
                .saturating_add(self.rows.traits.len())
                .saturating_mul(4 * size_of::<s::HeaderContract>()),
        )
        .map_err(Error::unlocated)?;
        let take = |declaration: crate::support::crates::owner_catalog::DeclarationRef,
                    completed: &mut Vec<Option<s::HeaderContract>>| {
            completed
                .get_mut(declaration.0 as usize)
                .and_then(Option::take)
                .ok_or_else(|| {
                    Error::unlocated("source header lacks positive attribute completion")
                })
        };
        let functions = self
            .rows
            .functions
            .iter()
            .map(|header| take(header.declaration, &mut completed))
            .collect::<Result<_, _>>()?;
        let traits = self
            .rows
            .traits
            .iter()
            .map(|header| take(header.declaration, &mut completed))
            .collect::<Result<_, _>>()?;
        if completed.iter().any(Option::is_some) {
            return Err(Error::unlocated(
                "source attribute completion has no admitted header",
            ));
        }
        Ok(s::HeaderAttributeColumns { functions, traits })
    }
    fn header_contract(
        &mut self,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        view: HeaderAttributeView<'_>,
    ) -> Result<s::HeaderContract, Error> {
        let source = view
            .origin
            .source()
            .ok_or_else(|| Error::unlocated("source contract completion lost its local origin"))?;
        let declaration_site = self
            .site(declaration, source.declaration)
            .map_err(Error::unlocated)?;
        let must_use = match view.must_use {
            ContractKnowledge::Unknown => {
                return Err(Error::at(
                    source.owner.file,
                    source.span.start,
                    "source must_use contract producer is incomplete",
                ));
            }
            ContractKnowledge::KnownAbsent => None,
            ContractKnowledge::KnownPresent(value) => {
                let owner = value
                    .origin
                    .source()
                    .ok_or_else(|| Error::unlocated("source must_use origin is imported"))?;
                if value
                    .reason
                    .is_some_and(|reason| reason.len() > s::MAX_REASON_BYTES)
                {
                    return Err(Error::at(
                        owner.owner.file,
                        owner.span.start,
                        "source must_use reason exceeds transport profile limit",
                    ));
                }
                self.reserve(value.reason.map_or(0, str::len))
                    .map_err(Error::unlocated)?;
                self.charge(value.reason.map_or(1, str::len))
                    .map_err(Error::unlocated)?;
                Some(s::MustUseContract {
                    owner_site: self
                        .site(declaration, owner.owner)
                        .map_err(Error::unlocated)?,
                    reason: value.reason.map(str::to_owned),
                })
            }
        };
        let inline = match view.inline {
            ContractKnowledge::Unknown => {
                return Err(Error::at(
                    source.owner.file,
                    source.span.start,
                    "source inline contract producer is incomplete",
                ));
            }
            ContractKnowledge::KnownAbsent => None,
            ContractKnowledge::KnownPresent(value) => {
                let owner = value
                    .origin
                    .source()
                    .ok_or_else(|| Error::unlocated("source inline origin is imported"))?;
                Some(s::InlineContract {
                    owner_site: self
                        .site(declaration, owner.owner)
                        .map_err(Error::unlocated)?,
                    hint: match value.hint {
                        crate::sema::inlining::Hint::Hint => s::InlineHint::Hint,
                        crate::sema::inlining::Hint::Always => s::InlineHint::Always,
                        crate::sema::inlining::Hint::Never => s::InlineHint::Never,
                    },
                    applicability: value.applicability,
                })
            }
        };
        Ok(s::HeaderContract {
            declaration,
            declaration_site,
            must_use,
            inline,
        })
    }
}

impl Stage {
    pub(crate) fn require_attribute_context(
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
        if self.definitions.len() != report.resolution.definitions.len() {
            return Err(Error::unlocated(
                "source attribute stage belongs to another declaration arena",
            ));
        }
        Ok(())
    }
    pub(crate) fn selected_header(
        &self,
        report: &Report,
        facts: &Declarations,
        index: usize,
    ) -> Result<Option<(DefId, NodeKey)>, Error> {
        let Some(row) = self.definitions[index] else {
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
        let staged = self
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
pub(crate) fn require_defaultness_transport(
    report: &Report,
    declaration: NodeKey,
) -> Result<(), Error> {
    let Some(defaultness) = report.active_defaultness(declaration)? else {
        return Ok(());
    };
    if defaultness.node != declaration.node {
        return Err(Error::at(
            declaration.file,
            defaultness.span.start,
            "source defaultness header names another declaration",
        ));
    }
    Err(report.diagnostic(Error::at(
        declaration.file,
        defaultness.span.start,
        "source defaultness contract transport is not implemented",
    )))
}
