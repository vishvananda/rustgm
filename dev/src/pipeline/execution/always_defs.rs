//! Actual Always definitions share the existing constant-provider template owner.
use super::{BodyKey, Context, Declarations, Engine, Error, Owner, Report, Template};
use crate::{
    pipeline::original_contexts::Original,
    sema::{function_context::FunctionContext, model::Function},
    support::retained::Growth,
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    function: &Function,
    body: &crate::body::ir::Body,
) -> Result<(), Error> {
    let source = function
        .body
        .ok_or_else(|| Error::unlocated("comptime definition has no actual body"))?;
    if facts
        .source_function_context(report, function.definition, source)
        .map_err(Error::unlocated)?
        != FunctionContext::Comptime
        || body.source.source() != Some(source)
        || body.instance.is_some()
        || !body.const_context
        || !matches!(body.owner,Owner::Function{definition,signature,execution:None}
            if definition==function.definition && signature==function.signature)
    {
        return Err(report.body_error(
            source,
            "comptime defining CFG changed its actual original owner or context",
        ));
    }
    Ok(())
}
impl Context {
    /// None denotes an already checked immutable component/concrete original.
    pub(crate) fn take_comptime_definition(
        &self,
        report: &Report,
        facts: &mut Declarations,
        function: &Function,
    ) -> Result<Option<Original>, Error> {
        self.engine
            .try_borrow_mut()
            .map_err(|_| {
                Error::unlocated("constant provider is active during defining-body request")
            })?
            .take_comptime_definition(report, facts, function)
    }
    pub(crate) fn return_comptime_definition(
        &self,
        report: &Report,
        facts: &mut Declarations,
        original: Original,
    ) -> Result<(), Error> {
        self.engine
            .try_borrow_mut()
            .map_err(|_| {
                Error::unlocated("constant provider is active during defining-body publication")
            })?
            .return_comptime_definition(report, facts, original)
    }
}
impl Engine {
    fn take_comptime_definition(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        function: &Function,
    ) -> Result<Option<Original>, Error> {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        let key = BodyKey::Declaration(function.definition);
        if let Some(original) = self.component_original(key) {
            validate(report, facts, function, &original.template.body)?;
            Original::validate_parts_charged(facts, &original.template.body, &original.contract)
                .map_err(Error::unlocated)?;
            crate::body::constants::eligibility_charged(report, facts, &original.template.body)?;
            return Ok(None);
        }
        if let Some(body) = self.concrete.get(&key) {
            validate(report, facts, function, body)?;
            crate::body::constants::eligibility_charged(report, facts, body)?;
            return Ok(None);
        }
        let template = if let Some(template) = self.originals.remove(&function.definition) {
            validate(report, facts, function, &template.body)?;
            crate::body::constants::eligibility_charged(report, facts, &template.body)?;
            template
        } else {
            self.construction
                .function_charged(report, facts, function)?
        };
        validate(report, facts, function, &template.body)?;
        let contract = self
            .contracts
            .function(facts, function)
            .map_err(Error::unlocated)?;
        let bytes = contract.retained_bytes();
        reserve(facts, 1, bytes)?;
        let contract = contract.clone();
        Original::validate_parts_charged(facts, &template.body, &contract)
            .map_err(Error::unlocated)?;
        Ok(Some(Original { template, contract }))
    }
    fn return_comptime_definition(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        original: Original,
    ) -> Result<(), Error> {
        let Owner::Function { definition, .. } = original.template.body.owner else {
            return Err(Error::unlocated(
                "comptime defining publication requires a function",
            ));
        };
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .copied()
            .ok_or_else(|| Error::unlocated("comptime defining publication lost its function"))?;
        validate(report, facts, &function, &original.template.body)?;
        Original::validate_parts_charged(facts, &original.template.body, &original.contract)
            .map_err(Error::unlocated)?;
        if self.originals.contains_key(&definition)
            || self
                .concrete
                .contains_key(&BodyKey::Declaration(definition))
            || self.component_contains(BodyKey::Declaration(definition))
        {
            return Err(report.body_error(
                original.template.body.source,
                "duplicate comptime defining owner",
            ));
        }
        self.reserve_template(report, facts, definition, &original.template)?;
        let growth = Growth::map::<crate::resolve::model::DefId, Template>(
            self.originals.len(),
            self.originals.capacity(),
            1,
        );
        reserve(facts, growth.work.saturating_add(1), growth.bytes)?;
        self.originals
            .try_reserve(1)
            .map_err(|_| Error::unlocated("comptime defining retention failed"))?;
        self.originals.insert(definition, original.template);
        Ok(())
    }
}
fn reserve(facts: &mut Declarations, steps: usize, bytes: usize) -> Result<(), Error> {
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
    facts.constant_work.template_bytes = facts.constant_work.template_bytes.saturating_add(bytes);
    if facts.constant_work.template_bytes > 64 * 1024 * 1024 {
        return Err(Error::unlocated(
            "comptime defining storage exceeds constant-provider budget",
        ));
    }
    Ok(())
}
