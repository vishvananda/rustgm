//! Demanded predicate uses share the ordinary immutable macro-frame authority.
use super::{Engine, Error, FileId, Input, Location, MacroContext, Report, Sources, builtin};
use crate::{
    expand::config::{CfgGate, CfgUse},
    resolve::bindings::KnownFeature,
};

impl<S: Sources> Engine<'_, S> {
    pub(super) fn cfg_expansion(
        &mut self,
        file: FileId,
        input: &Input,
        grammar: MacroContext,
        original: crate::syntax::token_tree::Span,
    ) -> Result<builtin::Expansion, Error> {
        let answer = {
            let package = &self.loader.package;
            let view = self
                .captures
                .view(|id| package.units.get(id.0 as usize).map(|u| u.source.text()));
            let inputs = crate::syntax::edition_context::WithEditions::new(&view, |token| {
                crate::resolve::unit::editions::token(file, token, |id| {
                    package.units.get(id.0 as usize)
                })
            });
            builtin::cfg_evaluation(
                package.unit(file).source.text(),
                input,
                &inputs,
                self.config,
            )
            .map_err(|e| {
                Error::at(
                    file,
                    if e.offset == 0 {
                        original.start
                    } else {
                        e.offset
                    },
                    e.message,
                )
            })?
        };
        self.check_cfg_uses(file, &answer.uses)?;
        builtin::cfg_value(answer.value, grammar, self.config).map_err(|e| {
            Error::at(
                file,
                if e.offset == 0 {
                    original.start
                } else {
                    e.offset
                },
                e.message,
            )
        })
    }

    pub(super) fn check_cfg_uses(&mut self, file: FileId, uses: &[CfgUse]) -> Result<(), Error> {
        if !self.loader.package.requested_cfg_ub_checks() {
            let view = super::permission_frames::FrameView {
                package: &self.loader.package,
                definitions: &self.definitions,
                invocations: &self.invocations,
                facts: &self.paths.state.facts,
            };
            let mut query = super::permission_session::Query {
                view,
                memo: &mut self.cfg_memo,
            };
            for &used in uses {
                let site = site(file, used);
                if !query.allows(site, &KnownFeature::CfgUbChecks.name())? {
                    return Err(missing(site));
                }
            }
        }
        Ok(())
    }
}

fn site(file: FileId, used: CfgUse) -> Location {
    match used.gate {
        CfgGate::UbChecks => Location {
            file,
            span: used.span,
        },
    }
}
fn missing(site: Location) -> Error {
    Error::at(
        site.file,
        site.span.start,
        "cfg(ub_checks) requires cfg_ub_checks or its actual macro allowance",
    )
}

pub(super) fn validate_attributes(report: &Report) -> Result<(), Error> {
    let mut session = report.macro_allowances();
    let mut demand: super::removed_functions::ConfiguredDemandSession<'_> =
        report.configured_demand_session();
    for (index, unit) in report.package.units.iter().enumerate() {
        let file = FileId(u32::try_from(index).map_err(|_| Error::unlocated("cfg file overflow"))?);
        for (ordinal, _) in unit
            .configured
            .cfg_uses
            .iter()
            .flat_map(|uses| uses.iter())
            .enumerate()
        {
            let used = demand.checked(file, ordinal)?;
            let location = used.location();
            match demand.classify(&used)? {
                super::removed_functions::DemandDisposition::RemovedTestBody(receipt) => {
                    let _ = receipt;
                    continue;
                }
                super::removed_functions::DemandDisposition::Demanded => {}
            }
            if !used.in_attribute_span() {
                return Err(Error::at(
                    file,
                    location.span.start,
                    "cfg use changed its original attribute/capture",
                ));
            }
            if !report.package.requested_cfg_ub_checks()
                && !session.allows(location, &KnownFeature::CfgUbChecks.name())?
            {
                return Err(missing(location));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/cfg_gate_work.rs"]
mod tests;
