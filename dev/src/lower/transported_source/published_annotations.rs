//! One selected-header walk emits positively completed cold contract columns.
use super::{Declarations, Report, Source, publication::selected_header, s};
use crate::{
    resolve::package::Error,
    sema::header_attributes::{ContractKnowledge, HeaderAttributeView},
};

impl Source {
    pub(super) fn collect_header_columns(
        &mut self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<s::HeaderAttributeColumns, Error> {
        self.require_attribute_context(report, facts)?;
        let count = self.stage.rows.declarations.len();
        self.stage
            .reserve(
                count.saturating_mul(size_of::<Option<s::HeaderContract>>())
                    + size_of::<s::HeaderAttributeColumns>(),
            )
            .map_err(Error::unlocated)?;
        let mut completed: Vec<Option<s::HeaderContract>> = (0..count).map(|_| None).collect();
        for index in 0..self.stage.definitions.len() {
            self.stage.charge(1).map_err(Error::unlocated)?;
            let Some((definition, declaration)) =
                selected_header(&self.stage, report, facts, index)?
            else {
                continue;
            };
            let row = self.stage.definitions[index]
                .ok_or_else(|| Error::unlocated("selected attribute declaration disappeared"))?;
            let view = facts.header_attribute_contracts(report, definition)?;
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
        self.stage
            .reserve(
                self.stage
                    .rows
                    .functions
                    .len()
                    .saturating_add(self.stage.rows.traits.len())
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
            .stage
            .rows
            .functions
            .iter()
            .map(|header| take(header.declaration, &mut completed))
            .collect::<Result<_, _>>()?;
        let traits = self
            .stage
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
            .stage
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
                self.stage
                    .reserve(value.reason.map_or(0, str::len))
                    .map_err(Error::unlocated)?;
                self.stage
                    .charge(value.reason.map_or(1, str::len))
                    .map_err(Error::unlocated)?;
                Some(s::MustUseContract {
                    owner_site: self
                        .stage
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
                        .stage
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
