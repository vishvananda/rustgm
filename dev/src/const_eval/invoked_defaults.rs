//! Reached default calls retain their checked caller and select const impls afresh.
use super::{
    Frame, Machine, State,
    code::{Catalog, Code},
};
use crate::{
    body::ir::{Action, Rvalue},
    sema::model::Declarations,
    types::{Kind, callable::Convention, instance::Instance},
};

fn imported_caller(facts: &Declarations, body: &Code) -> Result<(), &'static str> {
    let crate::body::ir::Owner::Function {
        definition,
        signature,
        execution,
    } = body.owner
    else {
        return Err("imported const default caller lost its materialized function owner");
    };
    let header = facts
        .declaration_header(definition)?
        .ok_or("imported const default caller lost its canonical declaration")?;
    let matched = match body.instance {
        Some(instance) => {
            execution.is_some()
                && matches!(
                    facts.types.kind(instance.ty()),
                    Some(Kind::FunctionItem { item, signature: actual, .. })
                        if item.0 == definition.0 && actual == signature
                )
        }
        None => execution.is_none() && header.signature == signature,
    };
    if header.body != Some(body.source)
        || !(facts.is_const_function(definition) || facts.imported_const_default(definition)?)
        || !matched
    {
        return Err("imported const default caller changed its authenticated body or instance");
    }
    Ok(())
}

fn validate(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    charge: &mut impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    charge(32)?;
    if body.source.source().is_some() {
        body.owner
            .source_caller(facts, body.source, body.instance)?;
    } else {
        imported_caller(facts, body)?;
    }
    let block = body
        .blocks
        .get(frame.block.0 as usize)
        .ok_or("const default caller lost its checked block")?;
    if !body.const_context || position < block.start || position >= block.end {
        return Err("const default call is outside its checked const block");
    }
    let Some(Action::Evaluate(value)) = body.statements.get(position as usize).map(|s| s.action)
    else {
        return Err("const default requires its actual evaluated statement");
    };
    let value = body
        .values
        .get(value.0 as usize)
        .ok_or("const default lost its evaluated destination")?;
    let Rvalue::Call {
        callee,
        signature,
        arguments,
    } = value.kind
    else {
        return Err("const default requires its actual direct Call");
    };
    let callee_value = body
        .values
        .get(callee.0 as usize)
        .ok_or("const default lost its evaluated callee")?;
    let Rvalue::Function(definition) = callee_value.kind else {
        return Err("const default requires its actual function item operand");
    };
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(callee_value.ty) else {
        return Err("const default operand lost its complete callable");
    };
    if definition.0 != item.0
        || body.callable_target(Instance::new(&facts.types, callee_value.ty)?) != target
        || Machine::value(frame, callee)
            .map_err(|_| "const default callee was not evaluated")?
            .as_function()
            != Some(target)
    {
        return Err("const default changed its actual selected callable");
    }
    let Some(Kind::Function {
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST,
        parameters,
        result,
    }) = facts.types.kind(signature)
    else {
        return Err("const default lost its opened safe Rust call signature");
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("const default lost its parameter list")?;
    let arguments = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("const default lost its argument range")?;
    if result != value.ty || parameters.len() != arguments.len() {
        return Err("const default changed its result or argument count");
    }
    charge(arguments.len().saturating_mul(3))?;
    for (&parameter, &argument) in parameters.iter().zip(arguments) {
        if body
            .values
            .get(argument.0 as usize)
            .is_none_or(|v| v.ty != parameter)
            || Machine::value(frame, argument).is_err()
        {
            return Err("const default changed its actual opened argument");
        }
    }
    Ok(())
}

impl State {
    pub(crate) fn check_reached_default(
        &mut self,
        facts: &Declarations,
        catalog: &Catalog,
        target: Instance,
        steps: usize,
    ) -> Result<(), &'static str> {
        let frame = self
            .frames
            .last()
            .ok_or("const default has no reached caller")?;
        catalog.validate_entry(frame.body)?;
        let body = catalog
            .code(frame.body)
            .ok_or("const default caller Code disappeared")?;
        validate(facts, body, frame, frame.next, target, &mut |count| {
            self.work
                .charge(count, steps)
                .map_err(crate::sema::values::Failure::message)
        })
    }
}

impl Machine<'_, '_> {
    pub(super) fn check_default_call(
        &mut self,
        body: &Code,
        frame: &Frame,
        target: Instance,
        form: super::function_calls::Form,
    ) -> Result<bool, super::Failure> {
        self.charge(8)?;
        if self
            .evaluator
            .facts()
            .const_default_member(target)?
            .is_none()
        {
            return Ok(false);
        }
        if !matches!(form, super::function_calls::Form::Direct) {
            return Err("const default indirect invocation is not implemented".into());
        }
        self.bodies.validate_entry(frame.body)?;
        let position = frame
            .next
            .checked_sub(1)
            .ok_or("const default has no current statement")?;
        validate(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            &mut |count| {
                self.work
                    .charge(count, self.limits.steps)
                    .map_err(crate::sema::values::Failure::message)
            },
        )?;
        self.evaluator.check_const_default(target)?;
        Ok(true)
    }
}
