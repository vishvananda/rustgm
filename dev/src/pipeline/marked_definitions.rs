//! A real marked definition requests ordinary const CFG validation, never execution.
use super::{
    Declarations, DefId, Demand, Error, Function, Report, execution, original_contexts::Original,
};
use crate::{body, sema::function_context::FunctionContext, support::retained::Growth};

impl Demand {
    pub(super) fn defining_function(
        facts: &Declarations,
        definition: DefId,
    ) -> Result<Function, Error> {
        facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .copied()
            .filter(|f| f.definition == definition)
            .ok_or_else(|| Error::unlocated("defining request lost its actual function"))
    }
    pub(super) fn defining_original(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        function: &Function,
        context: Option<&execution::Context>,
    ) -> Result<Option<Original>, Error> {
        if facts.is_comptime_function(function.definition)
            && let Some(context) = context
        {
            return context.take_comptime_definition(report, facts, function);
        }
        let template = if facts.is_comptime_function(function.definition) {
            self.prepared
                .templates
                .function_charged(report, facts, function)?
        } else {
            self.prepared.templates.function(report, facts, function)?
        };
        let contract = self
            .prepared
            .contracts
            .get_or_insert_with(Default::default)
            .function(facts, function)
            .map_err(Error::unlocated)?
            .clone();
        Ok(Some(Original { template, contract }))
    }
    pub(super) fn complete_comptime(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        definition: DefId,
        context: Option<&execution::Context>,
    ) -> Result<(), Error> {
        let function = Self::defining_function(facts, definition)?;
        let source = function
            .body
            .ok_or_else(|| Error::unlocated("marked defining body is absent"))?;
        if !facts.typed_function_bodies.contains(&definition)
            || facts.patterns.pending_bodies.contains(&source)
            || facts
                .source_function_context(report, definition, source)
                .map_err(Error::unlocated)?
                != FunctionContext::Comptime
        {
            return Err(
                report.body_error(source, "comptime defining body is not genuinely complete")
            );
        }
        if let Some(original) = self.defining_original(report, facts, &function, context)? {
            self.retain_defining_original(report, facts, original, context)?;
        }
        Ok(())
    }
    pub(super) fn retain_defining_original(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        original: Original,
        context: Option<&execution::Context>,
    ) -> Result<(), Error> {
        let body::ir::Owner::Function { definition, .. } = original.template.body.owner else {
            return Err(Error::unlocated("defining retention requires a function"));
        };
        if facts.is_comptime_function(definition) {
            if let Some(context) = context {
                return context.return_comptime_definition(report, facts, original);
            }
            if !body::templates::owns(facts, definition) {
                let key = body::ir::BodyKey::Declaration(definition);
                if self.prepared.concrete.contains_key(&key) {
                    return Err(Error::unlocated("duplicate marked defining CFG"));
                }
                let growth = Growth::map::<body::ir::BodyKey, body::ir::Body>(
                    self.prepared.concrete.len(),
                    self.prepared.concrete.capacity(),
                    1,
                );
                let bytes = original
                    .template
                    .retained_bytes()
                    .saturating_mul(2)
                    .saturating_add(growth.bytes);
                facts.constant_work.template_bytes =
                    facts.constant_work.template_bytes.saturating_add(bytes);
                facts.body_work.canonical_steps = facts
                    .body_work
                    .canonical_steps
                    .saturating_add(growth.work.saturating_add(1));
                if facts.constant_work.template_bytes > 64 * 1024 * 1024 {
                    return Err(Error::unlocated(
                        "marked defining retention exceeds storage budget",
                    ));
                }
                self.prepared
                    .concrete
                    .try_reserve(1)
                    .map_err(|_| Error::unlocated("marked defining retention failed"))?;
                self.prepared.concrete.insert(key, original.template.body);
                return Ok(());
            }
        }
        self.prepared
            .templates
            .retain(original.template)
            .map_err(Error::unlocated)
    }
}
