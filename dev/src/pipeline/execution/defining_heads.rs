//! Only fully proved original defining bodies may publish hidden storage heads.
use super::{CheckedInput, Contract, Declarations, Engine, Error, Report, Template, Templates};
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
        purpose: super::ProofPurpose,
    ) -> Result<CheckedInput, Error> {
        let body = &template.body;
        let definition = match body.owner {
            Owner::Function { definition, .. } => definition,
            Owner::AsyncBody { .. } => {
                crate::body::coroutines::original(facts, body).map_err(identity_error)?
            }
            _ => {
                return Err(identity_error(
                    "defining proof is not an authored function body",
                ));
            }
        };
        if !facts.opaques.owners.contains_key(&definition) {
            return self.prove(report, facts, template, contract, purpose);
        }
        let source_arena = if let Owner::AsyncBody { future, .. } = body.owner {
            Some(
                facts
                    .types
                    .future_proof_arena(future)
                    .map_err(identity_error)?,
            )
        } else {
            None
        };
        Original::validate_parts_charged(facts, body, contract).map_err(identity_error)?;
        let mut contract = std::borrow::Cow::Borrowed(contract);
        let mut regions = Self::source_regions(report, facts, body, &contract)?;
        let checked = self.prove_loans(
            report,
            facts,
            body,
            &template.destruction,
            &mut super::LoanInput {
                parameters: &template.environment,
                contract: &mut contract,
                regions: &mut regions,
                purpose,
            },
        )?;
        if matches!(body.owner, Owner::AsyncBody { .. }) {
            let arena = source_arena.as_ref().ok_or_else(|| {
                identity_error("async source proof lost its actual store witness")
            })?;
            facts
                .infer_checked_async_future(definition, arena)
                .map_err(|e| report.body_error(body.source, e))?;
        }
        facts
            .close_opaque_results(definition, &regions.hidden_regions)
            .map_err(|e| report.body_error(body.source, e))?;
        if matches!(body.owner, Owner::AsyncBody { .. }) {
            // The real original has been proved, but it is never standalone code.
            // Generated entries still require their actual protocol and checked family.
            return Ok(CheckedInput {
                retained: None,
                code: None,
            });
        }
        let (code, uses) = self
            .finish_code(facts, body, &contract, &regions, purpose, checked)
            .map_err(|e| report.body_error(body.source, e))?;
        let retained = if uses.needed {
            Some(
                self.retained_source(
                    code.as_ref(),
                    std::sync::Arc::new(contract.into_owned()),
                    uses,
                )
                .map_err(|e| report.body_error(body.source, e))?,
            )
        } else {
            None
        };
        Ok(CheckedInput { retained, code })
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
        let (definition, original, contract, regions) = match body.owner {
            Owner::Function { definition, .. } => {
                if !facts.opaques.owners.contains_key(&definition) {
                    continue;
                }
                let (original, contract, regions) = checked
                    .function_proof(ItemId(definition.0))
                    .ok_or_else(|| identity_error("missing checked defining function"))?;
                (definition, original, contract, regions)
            }
            Owner::AsyncBody { identity, .. } => {
                checked
                    .require_future_store(&facts.types)
                    .map_err(identity_error)?;
                let definition =
                    crate::body::coroutines::original(facts, body).map_err(identity_error)?;
                let (original, contract, regions) = checked
                    .async_body(identity)
                    .ok_or_else(|| identity_error("missing checked deferred AsyncBody"))?;
                (definition, original, contract, regions)
            }
            _ => continue,
        };
        if !facts.opaques.owners.contains_key(&definition) {
            continue;
        }
        let error = identity_error;
        if !std::ptr::eq(original, template) || regions.source != body.source {
            return Err(error("defining proof belongs to another original"));
        }
        Original::validate_parts_charged(facts, body, contract).map_err(error)?;
    }
    // A Checked family seals the CFG proofs, but does not validate declaration
    // identity. Every original pairing must pass before any head is published.
    for template in &templates.bodies {
        let (definition, regions) = match template.body.owner {
            Owner::Function { definition, .. } => {
                if !facts.opaques.owners.contains_key(&definition) {
                    continue;
                }
                let (_, _, regions) = checked
                    .function_proof(ItemId(definition.0))
                    .ok_or_else(|| identity_error("missing checked defining function"))?;
                (definition, regions)
            }
            Owner::AsyncBody { identity, .. } => {
                let definition = crate::body::coroutines::original(facts, &template.body)
                    .map_err(identity_error)?;
                let (_, _, regions) = checked
                    .async_body(identity)
                    .ok_or_else(|| identity_error("missing checked deferred AsyncBody"))?;
                (definition, regions)
            }
            _ => continue,
        };
        if !facts.opaques.owners.contains_key(&definition) {
            continue;
        }
        if matches!(template.body.owner, Owner::AsyncBody { .. }) {
            let arena = checked.future_store(&facts.types).map_err(identity_error)?;
            facts
                .infer_checked_async_future(definition, arena)
                .map_err(|e| report.body_error(template.body.source, e))?;
        }
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
