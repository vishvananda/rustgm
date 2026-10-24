//! Actual function entry shares const gates without treating Invoke as an intrinsic call.
use super::{Constant, Failure, Frame, Machine, Requirement, code::Code};
use crate::{
    resolve::model::DefId,
    types::{Kind, instance::Instance},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Form {
    Direct,
    Invoke,
    Selected { selector: Instance },
}
pub(super) enum Outcome {
    Constant(Constant),
    Enter(usize),
    Pending,
}
impl Machine<'_, '_> {
    pub(super) fn function_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        target: Instance,
        form: Form,
    ) -> Result<Outcome, Failure> {
        self.evaluator.check_source_call_policy()?;
        let Some(Kind::FunctionItem { item, .. }) = self.evaluator.facts().types.kind(target.ty())
        else {
            return Err("constant callable lost its item identity".into());
        };
        let definition = DefId(item.0);
        if self
            .evaluator
            .facts()
            .associated
            .trait_owner(definition)
            .is_some()
        {
            self.charge(5)?;
            self.evaluator
                .facts()
                .check_const_member_execution(definition)?;
        }
        if !self.evaluator.facts().is_const_function(definition) {
            return Err("non-const calls are not allowed during constant execution".into());
        }
        crate::sema::compiler_items::check_const_drop_glue(self.evaluator.facts(), target)?;
        if let Some(catalog) = self.evaluator.facts().source_bindings.as_deref() {
            if matches!(form, Form::Selected { .. })
                && catalog.intrinsic_kind(definition)?.is_some()
            {
                return Err(
                    "source intrinsic selected callback execution is not implemented".into(),
                );
            }
            if catalog.requires_const_payment(definition)? {
                if matches!(form, Form::Invoke) {
                    return Err("conditional Invoke target payment is not implemented".into());
                }
                if let Some(index) = self.consume_const_call(body, frame, target, form)? {
                    if matches!(form, Form::Selected { .. }) {
                        self.validate_invoked_entry(index, target)?;
                        self.validate_selected_entry(index, target)?;
                    }
                    return Ok(Outcome::Enter(index));
                }
                self.waiting = Some(Requirement::FunctionBody(target));
                return Ok(Outcome::Pending);
            }
            if self.paid_call.is_some() {
                return Err("source const payment changed its declaration effect".into());
            }
            catalog.check_const_call(definition)?;
            if matches!(form, Form::Invoke) && catalog.intrinsic_kind(definition)?.is_some() {
                return Err("source intrinsic Invoke execution is not implemented".into());
            }
        }
        if matches!(form, Form::Direct) {
            match self.source_intrinsic_call(body, frame, target)? {
                super::executed_calls::Answer::Ready(value) => return Ok(Outcome::Constant(value)),
                super::executed_calls::Answer::Pending => return Ok(Outcome::Pending),
                super::executed_calls::Answer::Ordinary => {}
            }
        }
        let selected = self.bodies.functions.get(&target).copied().or_else(|| {
            if matches!(form, Form::Direct) {
                self.bodies.definitions.get(&definition).copied()
            } else {
                None
            }
        });
        let Some(index) = selected else {
            self.waiting = Some(Requirement::FunctionBody(target));
            return Ok(Outcome::Pending);
        };
        if !matches!(form, Form::Direct) {
            self.validate_invoked_entry(index, target)?;
        }
        if matches!(form, Form::Selected { .. }) {
            self.validate_selected_entry(index, target)?;
        }
        Ok(Outcome::Enter(index))
    }
    fn validate_selected_entry(&self, index: usize, target: Instance) -> Result<(), &'static str> {
        let facts = self.evaluator.facts();
        let code = self
            .bodies
            .code(index)
            .ok_or("selected const callback code disappeared")?;
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(target.ty())
        else {
            return Err("selected const callback lost its complete function item");
        };
        let definition = DefId(item.0);
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("selected const callback has no actual declaration")?;
        let Some((public, owned)) = code.owner.signatures() else {
            return Err("selected const callback has no paired callable signatures");
        };
        let Some(Kind::Function {
            bound_regions: 0,
            result,
            ..
        }) = facts.types.kind(owned)
        else {
            return Err("selected const callback has no exact opened execution signature");
        };
        if self.bodies.functions.get(&target) != Some(&index)
            || function.definition != definition
            || code.source.source() != Some(function.body.unwrap_or(function.node))
            || public != signature
            || result != code.result
            || code.owner.key(code.source) != crate::body::ir::BodyKey::Declaration(definition)
        {
            return Err("selected const callback changed source/owner/instance/signature/index");
        }
        Ok(())
    }
    fn validate_invoked_entry(&self, index: usize, target: Instance) -> Result<(), &'static str> {
        self.bodies.validate_entry(index)?;
        let code = self
            .bodies
            .code(index)
            .ok_or("Invoke callee code disappeared")?;
        let Some(Kind::FunctionItem {
            item,
            signature,
            arguments,
        }) = self.evaluator.facts().types.kind(target.ty())
        else {
            return Err("Invoke target lost its full instance");
        };
        let crate::body::ir::Owner::Function { definition, .. } = code.owner else {
            return Err("Invoke selected a non-function body");
        };
        let Some(Kind::Function { result, .. }) = self.evaluator.facts().types.kind(signature)
        else {
            return Err("Invoke target lost its signature");
        };
        if !code.const_context
            || definition != DefId(item.0)
            || code.result != result
            || code.owner.signatures().map(|(public, _)| public) != Some(signature)
            || code.instance.is_some_and(|instance| instance != target)
            || (code.instance.is_none()
                && self
                    .evaluator
                    .facts()
                    .types
                    .arguments(arguments)
                    .is_none_or(|args| args.len() != 0))
        {
            return Err("Invoke selected another callable body or complete instance");
        }
        Ok(())
    }
}
