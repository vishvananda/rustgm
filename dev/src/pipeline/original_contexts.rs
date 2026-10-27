//! Original proof context is separate from every executable template lookup.
use crate::{
    body::{
        ir::{BodyKey, Owner},
        regions::Contract,
        templates::{Template, Templates},
    },
    sema::model::Declarations,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
const BYTES: usize = 64 * 1024 * 1024;
#[derive(Default)]
pub(crate) struct Originals {
    contexts: Vec<Template>,
    contracts: HashMap<BodyKey, Contract>,
    bytes: usize,
}

#[cfg(test)]
#[path = "../../tests/invariants/published_parents.rs"]
mod tests;
pub(crate) struct Original {
    pub template: Template,
    pub contract: Contract,
}
impl Original {
    /// Raw imported equality precedes every ephemeral caller-local transfer.
    pub(crate) fn prepare_imported(
        facts: &mut Declarations,
        body: &crate::body::ir::Body,
        contract: &Contract,
    ) -> Result<Contract, String> {
        if body.instance.is_some() || body.source.source().is_some() {
            return Err("imported proof input requires its immutable original".into());
        }
        Self::validate_parts_charged(facts, body, contract)?;
        if facts
            .constant_work
            .template_bytes
            .saturating_add(contract.retained_bytes())
            > 64 * 1024 * 1024
        {
            return Err("imported original proof input exceeds aggregate storage budget".into());
        }
        let mut prepared = contract.clone();
        crate::body::copy_requirements::prepare(facts, body, &mut prepared)?;
        prepared.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
        if facts
            .constant_work
            .template_bytes
            .saturating_add(prepared.retained_bytes())
            > 64 * 1024 * 1024
        {
            return Err("imported original proof input exceeds aggregate storage budget".into());
        }
        Ok(prepared)
    }
    pub(crate) fn prepare_selected(
        facts: &mut Declarations,
        original: &crate::body::transported_originals::Original,
        preparation: &mut dyn crate::sema::dispatch::Preparation,
    ) -> Result<Contract, crate::resolve::package::Error> {
        use crate::resolve::package::Error;
        let body = &original.template().body;
        let crate::body::ir::Owner::Function { definition, .. } = body.owner else {
            return Err(Error::unlocated(
                "selected original requires its actual function owner",
            ));
        };
        let mut steps = 0;
        let validated = preparation.with_original(facts, definition, &mut |facts, view| {
            crate::body::transported_originals::validate_selected(facts, view, original, &mut steps)
                .map_err(Error::unlocated)
        });
        let charged = preparation.prepay(facts, steps, 0);
        validated.and(charged)?;
        let raw = original.contract();
        preparation.prepay(facts, 1, raw.retained_bytes())?;
        let mut prepared = raw.clone();
        crate::body::copy_requirements::prepare_original_selected(
            facts,
            body,
            &mut prepared,
            preparation,
        )
        .map_err(|failure| match failure {
            crate::sema::dispatch::Failure::Invalid(message) => Error::unlocated(message),
            crate::sema::dispatch::Failure::Preparation(error) => error,
        })?;
        prepared
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(Error::unlocated)?;
        Ok(prepared)
    }
    pub fn validate(&self, facts: &Declarations) -> Result<(), &'static str> {
        Self::validate_parts(facts, &self.template.body, &self.contract)?;
        if let Owner::Function { definition, .. } = self.template.body.owner
            && let Some(environment) = facts
                .function_header(definition)?
                .and_then(|h| h.environment)
            && !Arc::ptr_eq(environment, &self.template.environment)
        {
            return Err("original function changed its exact declaration environment");
        }
        Ok(())
    }
    pub(super) fn validate_parts(
        facts: &Declarations,
        body: &crate::body::ir::Body,
        contract: &Contract,
    ) -> Result<(), &'static str> {
        Self::validate_parts_counted(facts, body, contract, &mut 0)
    }
    pub(super) fn validate_parts_charged(
        facts: &mut Declarations,
        body: &crate::body::ir::Body,
        contract: &Contract,
    ) -> Result<(), &'static str> {
        let mut steps = 0;
        let result = Self::validate_parts_counted(facts, body, contract, &mut steps);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .checked_add(steps)
            .ok_or("original input validation work counter overflow")?;
        if steps > crate::support::crates::MAX_COMPONENTS {
            return Err("original input validation work budget exceeded");
        }
        result
    }
    fn validate_parts_counted(
        facts: &Declarations,
        body: &crate::body::ir::Body,
        contract: &Contract,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        if body.instance.is_some() || contract.key != body.key() || contract.source != body.source {
            return Err("original component contract belongs to another body");
        }
        if body.source.source().is_none() {
            if crate::body::transported_originals::validate_source_parts(
                facts, body, contract, steps,
            )? {
                return Ok(());
            }
            return Self::validate_legacy_imported_parts(facts, body, contract);
        }
        let source = body
            .source
            .source()
            .ok_or("source-free original requires its authenticated imported header")?;
        let valid = match body.owner {
            Owner::Function {
                definition,
                signature,
                execution: None,
            } => facts.function_header(definition)?.is_some_and(|header| {
                header.body == Some(body.source)
                    && header.signature == signature
                    && contract.signature == Some(signature)
                    && contract.universals == header.universals
                    && header.declaration.source().is_some()
            }),
            Owner::Initializer { definition, kind } => facts
                .initializer_index(definition)
                .and_then(|i| facts.initializers.get(i))
                .is_some_and(|i| {
                    i.semantic_source() == source
                        && i.ty == body.result
                        && contract.signature.is_none()
                        && contract.universals
                            == facts
                                .associated
                                .schema(definition)
                                .map_or(0, |s| s.captures)
                        && match (i.kind, kind) {
                            (
                                crate::sema::model::InitializerKind::Const,
                                crate::sema::model::InitializerKind::Const,
                            ) => true,
                            (
                                crate::sema::model::InitializerKind::Static { mutable: a },
                                crate::sema::model::InitializerKind::Static { mutable: b },
                            ) => a == b,
                            _ => false,
                        }
                }),
            Owner::Closure {
                identity,
                signature,
                execution,
            } => facts.closures.identity(identity).is_some_and(|c| {
                c.semantic_source() == source
                    && c.signature == signature
                    && c.execution.map(|e| e.signature) == execution
                    && contract.signature == Some(c.execution.map_or(signature, |e| e.signature))
                    && contract.universals == c.execution.map_or(c.universals, |e| e.universals)
            }),
            Owner::AnonymousConstant => facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(source))
                .is_some_and(|i| {
                    i.result == Some(body.result)
                        && i.universals == contract.universals
                        && contract.signature.is_none()
                }),
            _ => false,
        };
        let link = facts.nested_outputs.as_deref().and_then(|r| r.get(&source));
        if !valid
            || match (&contract.output, link) {
                (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
                (None, None) => false,
                _ => true,
            }
        {
            return Err("original component changed its typed owner or output link");
        }
        Ok(())
    }
    fn validate_legacy_imported_parts(
        facts: &Declarations,
        body: &crate::body::ir::Body,
        contract: &Contract,
    ) -> Result<(), &'static str> {
        let Owner::Function {
            definition,
            signature,
            execution: None,
        } = body.owner
        else {
            return Err("unsupported imported original owner category");
        };
        let header = facts
            .function_header(definition)?
            .ok_or("imported original header is absent")?;
        let environment = header
            .environment
            .ok_or("imported original environment is absent")?;
        let expected_inputs = crate::sema::declaration_inputs::own(
            &facts.generics,
            facts.crates.as_deref(),
            definition,
        )?
        .map(|input| input.value);
        let actual_environment = contract
            .builtins
            .as_deref()
            .and_then(|b| b.environment.as_ref());
        if header.body != Some(body.source)
            || header.signature != signature
            || header.declaration.source().is_some()
            || contract.signature != Some(signature)
            || contract.universals != header.universals
            || contract.output.is_some()
            || !matches!(facts.types.kind(signature), Some(crate::types::Kind::Function { result, .. }) if result == body.result)
            || contract.parameters != facts.generics.parameter_ids(&facts.types, definition)?
            || !contract
                .type_assumptions
                .iter()
                .copied()
                .eq(expected_inputs)
            || actual_environment.is_none_or(|actual| !Arc::ptr_eq(actual, environment))
        {
            return Err(
                "imported original changed its exact header, environment or contract owner",
            );
        }
        Ok(())
    }
    fn candidate(&self, facts: &Declarations) -> bool {
        match self.template.body.owner {
            Owner::Function { definition, .. } => crate::body::templates::owns(facts, definition),
            Owner::Initializer { definition, .. } => {
                facts.associated.owner(definition).is_some()
                    || facts.generics.has_parameters(definition)
            }
            Owner::Closure { identity, .. } => facts
                .closures
                .identity(identity)
                .and_then(|c| c.owner)
                .is_some_and(|o| crate::body::templates::owns(facts, o)),
            Owner::AnonymousConstant => facts
                .inline_constants
                .as_deref()
                .and_then(|r| {
                    self.template
                        .body
                        .source
                        .source()
                        .and_then(|source| r.get(source))
                })
                .is_some_and(|i| {
                    i.evaluation == crate::sema::inline_constants::Evaluation::DeferredTemplate
                }),
            _ => false,
        }
    }
}
impl Originals {
    pub fn publish(
        &mut self,
        facts: &mut Declarations,
        prepared: &mut crate::body::templates::Prepared,
        rows: Vec<Original>,
    ) -> Result<(), &'static str> {
        let count = self.contracts.len().saturating_add(rows.len());
        let scratch = crate::support::retained::new_map::<BodyKey, ()>(rows.len()).saturating_add(
            rows.len()
                .saturating_mul(3 * size_of::<Template>() + size_of::<Contract>()),
        );
        let bytes = crate::support::retained::new_map::<BodyKey, Contract>(count)
            .saturating_sub(crate::support::retained::new_map::<BodyKey, Contract>(
                self.contracts.len(),
            ))
            // Each receiving template is either a map entry or a context Vec
            // slot. Charge geometric/minimum capacity before either grows.
            .saturating_add(crate::support::retained::new_map::<BodyKey, Template>(
                rows.len(),
            ))
            .saturating_add(
                rows.iter()
                    .map(|row| {
                        row.template
                            .retained_bytes()
                            .saturating_add(row.contract.retained_bytes())
                    })
                    .fold(0_usize, usize::saturating_add),
            );
        let total = facts.constant_work.template_bytes.saturating_add(bytes);
        if total.saturating_add(scratch) > BYTES {
            return Err("original component publication scratch exceeds storage budget");
        }
        let mut keys = HashSet::with_capacity(rows.len());
        for row in &rows {
            row.validate(facts)?;
            if self.contracts.contains_key(&row.contract.key) || !keys.insert(row.contract.key) {
                return Err("duplicate original component publication");
            }
        }
        let mut candidates = Vec::new();
        let mut contexts = Vec::new();
        let mut contracts = Vec::new();
        for row in rows {
            if row.candidate(facts) {
                candidates.push(row.template);
            } else {
                contexts.push(row.template);
            }
            contracts.push(row.contract);
        }
        // The receiving collection preflights every candidate before mutating it.
        prepared.retain_component(candidates)?;
        self.contexts.extend(contexts);
        for contract in contracts {
            self.contracts.insert(contract.key, contract);
        }
        self.bytes = self.bytes.saturating_add(bytes);
        facts.constant_work.template_bytes = total;
        Ok(())
    }
    pub fn apply(
        mut self,
        templates: &Templates,
        inputs: &mut [Contract],
    ) -> Result<crate::borrow::templates::Contexts, &'static str> {
        let scratch = crate::support::retained::new_map::<BodyKey, ()>(
            templates.bodies.len().saturating_add(self.contexts.len()),
        )
        .saturating_add(crate::support::retained::new_map::<BodyKey, ()>(
            inputs.len(),
        ))
        .saturating_add(
            self.contracts
                .len()
                .saturating_mul(3 * size_of::<Contract>() + size_of::<usize>()),
        );
        if self.bytes.saturating_add(scratch) > BYTES {
            return Err("original component input scratch exceeds storage budget");
        }
        let mut available: HashSet<_> = templates.bodies.iter().map(|t| t.body.key()).collect();
        for template in &self.contexts {
            if !available.insert(template.body.key()) {
                return Err("proof context duplicates an executable template owner");
            }
        }
        let mut seen = HashSet::with_capacity(inputs.len());
        let mut replacements = Vec::with_capacity(self.contracts.len());
        for (position, original) in inputs.iter().enumerate() {
            if !seen.insert(original.key) {
                return Err("duplicate original component input");
            }
            let Some(contract) = self.contracts.remove(&original.key) else {
                continue;
            };
            if !available.contains(&original.key)
                || original.source != contract.source
                || original.universals != contract.universals
                || original.signature != contract.signature
                || original.assumptions != contract.assumptions
                || original.parameters != contract.parameters
                || original.type_assumptions != contract.type_assumptions
                || original.require_result_formation != contract.require_result_formation
                || match (&original.output, &contract.output) {
                    (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
                    (None, None) => false,
                    _ => true,
                }
            {
                return Err("original component override changed its exact source header");
            }
            replacements.push((position, contract));
        }
        if !self.contracts.is_empty() {
            return Err("original component lost its source contract");
        }
        let mut source = Vec::with_capacity(replacements.len());
        for (position, contract) in replacements {
            source.push(std::mem::replace(&mut inputs[position], contract));
        }
        Ok(crate::borrow::templates::Contexts {
            bodies: Templates::source(self.contexts),
            source,
            imported: None,
        })
    }
}
