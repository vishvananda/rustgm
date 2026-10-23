//! Only fully proved original defining bodies may publish hidden storage heads.
use super::{Contract, Declarations, Engine, Error, Report, Template, Templates};
use crate::{body::ir::Owner, pipeline::original_contexts::Original, types::ItemId};

#[cfg(test)]
#[path = "../../../tests/invariants/defining_publication.rs"]
mod tests;

impl Engine {
    pub(super) fn prove_defining_original(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        template: &Template,
        contract: &Contract,
    ) -> Result<(), Error> {
        let body = &template.body;
        let Owner::Function { definition, .. } = body.owner else {
            return Err(identity_error("defining proof is not a function"));
        };
        if !facts.opaques.owners.contains_key(&definition) {
            return self.prove(report, facts, template, contract);
        }
        Original::validate_parts(facts, body, contract).map_err(identity_error)?;
        let regions = Self::source_regions(report, facts, body, contract)?;
        self.prove_loans(
            report,
            facts,
            body,
            &template.destruction,
            &template.environment,
            &regions,
        )?;
        facts
            .close_opaque_results(definition, &regions.hidden_regions)
            .map_err(|e| report.body_error(body.source, e))
    }
}

pub(super) fn close_component(
    report: &Report,
    facts: &mut Declarations,
    templates: &Templates,
    checked: &crate::borrow::templates::Checked<'_>,
) -> Result<(), Error> {
    if facts.opaques.owners.is_empty() {
        return Ok(());
    }
    for template in &templates.bodies {
        let body = &template.body;
        let Owner::Function { definition, .. } = body.owner else {
            continue;
        };
        if !facts.opaques.owners.contains_key(&definition) {
            continue;
        }
        let error = identity_error;
        let (original, contract, regions) = checked
            .function_proof(ItemId(definition.0))
            .ok_or_else(|| error("missing checked defining function"))?;
        if !std::ptr::eq(original, template) || regions.source != body.source {
            return Err(error("defining proof belongs to another original"));
        }
        Original::validate_parts(facts, body, contract).map_err(error)?;
    }
    // A Checked family seals the CFG proofs, but does not validate declaration
    // identity. Every original pairing must pass before any head is published.
    for template in &templates.bodies {
        let Owner::Function { definition, .. } = template.body.owner else {
            continue;
        };
        if !facts.opaques.owners.contains_key(&definition) {
            continue;
        }
        let (_, _, regions) = checked
            .function_proof(ItemId(definition.0))
            .ok_or_else(|| identity_error("missing checked defining function"))?;
        facts
            .close_opaque_results(definition, &regions.hidden_regions)
            .map_err(|e| report.body_error(template.body.source, e))?;
    }
    Ok(())
}

fn identity_error(message: &'static str) -> Error {
    // A claimed original source is untrusted until the complete header check.
    Error::at(crate::resolve::unit::FileId(0), 0, message)
}
