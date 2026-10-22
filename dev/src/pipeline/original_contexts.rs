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
    pub fn validate(&self, facts: &Declarations) -> Result<(), &'static str> {
        let body = &self.template.body;
        if body.instance.is_some()
            || self.contract.key != body.key()
            || self.contract.source != body.source
        {
            return Err("original component contract belongs to another body");
        }
        let valid = match body.owner {
            Owner::Function {
                definition,
                signature,
            } => facts
                .function_indices
                .get(&definition)
                .and_then(|&i| facts.functions.get(i))
                .is_some_and(|f| {
                    f.body == Some(body.source)
                        && f.signature == signature
                        && self.contract.signature == Some(signature)
                        && self.contract.universals
                            == facts
                                .lifetimes
                                .functions
                                .get(&definition)
                                .map_or(0, |s| s.total)
                }),
            Owner::Initializer { definition, kind } => facts
                .initializer_index(definition)
                .and_then(|i| facts.initializers.get(i))
                .is_some_and(|i| {
                    i.semantic_source() == body.source
                        && i.ty == body.result
                        && self.contract.signature.is_none()
                        && self.contract.universals
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
                c.semantic_source() == body.source
                    && c.signature == signature
                    && c.execution.map(|e| e.signature) == execution
                    && self.contract.signature
                        == Some(c.execution.map_or(signature, |e| e.signature))
                    && self.contract.universals
                        == c.execution.map_or(c.universals, |e| e.universals)
            }),
            Owner::AnonymousConstant => facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(body.source))
                .is_some_and(|i| {
                    i.result == Some(body.result)
                        && i.universals == self.contract.universals
                        && self.contract.signature.is_none()
                }),
            _ => false,
        };
        let link = facts
            .nested_outputs
            .as_deref()
            .and_then(|r| r.get(&body.source));
        if !valid
            || match (&self.contract.output, link) {
                (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
                (None, None) => false,
                _ => true,
            }
        {
            return Err("original component changed its typed owner or output link");
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
                .and_then(|r| r.get(self.template.body.source))
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
            bodies: Templates {
                bodies: self.contexts,
            },
            source,
        })
    }
}
