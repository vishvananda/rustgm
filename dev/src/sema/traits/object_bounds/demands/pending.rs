//! Completed declaration clauses are instantiated separately for each pending target.
use super::{Key, Template, cached, prepare, publish};
use crate::{
    sema::{generics::parameter_id, model::Declarations},
    types::{
        arguments::Argument as Canonical,
        infer::{arguments::Argument, objects::Principal},
        parameters::ParameterId,
        traits::Application,
    },
};

pub(in crate::sema) struct Prepared {
    pub template: Template,
    pub bindings: Vec<(ParameterId, Argument)>,
    pub steps: usize,
}

pub(in crate::sema) fn get(
    facts: &mut Declarations,
    principal: &Principal,
) -> Result<Prepared, &'static str> {
    let header = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.header(principal.item))
        .ok_or("missing pending object principal header")?;
    if header.parameters.len() != principal.arguments.len() {
        return Err("pending object principal arity mismatch");
    }
    let mut arguments = Vec::with_capacity(principal.arguments.len());
    let mut bindings = Vec::new();
    for (parameter, &actual) in header.parameters.iter().zip(&principal.arguments) {
        match (parameter.argument, actual) {
            (Canonical::Lifetime(_), Argument::Lifetime(region)) => {
                // Already inside the object's own binder, including its local slots.
                arguments.push(Canonical::Lifetime(region));
            }
            (formal @ Canonical::Type(_), Argument::Type(_))
            | (formal @ Canonical::Const(_), Argument::Const(_)) => {
                bindings.push((parameter_id(&facts.types, formal)?, actual));
                arguments.push(formal);
            }
            _ => return Err("pending object principal argument kind mismatch"),
        }
    }
    let arguments = facts.types.intern_arguments(&arguments)?;
    let application = facts.types.intern_trait_application(Application {
        item: principal.item,
        arguments,
    })?;
    let key = Key::Declared {
        principal: application,
        bound_regions: principal.bound_regions,
    };
    let (template, steps) = if let Some(template) = cached(facts, key)? {
        (template, 1)
    } else {
        let (template, steps) = prepare(facts, application, principal.bound_regions)?;
        publish(facts, key, &template, steps)?;
        (template, steps)
    };
    Ok(Prepared {
        template,
        bindings,
        steps: steps.saturating_add(principal.arguments.len()),
    })
}
