//! Substitute retained typed CFGs; the result still requires an instance proof.
mod contracts;
mod projections;
use super::{
    drops,
    ir::{Body, Owner, ProjectionKind, Rvalue},
    templates::Template,
};
use crate::{
    sema::model::Declarations,
    types::{
        Kind, arguments::Argument, destruction::NeedsDrop, instance::Instance, length::Length,
        substitute::Bound,
    },
};

#[derive(Clone, Copy)]
enum Premises {
    Required,
    Declared,
}

/// Concrete structural answers share one append-only type store and immutable
/// destructor registry. Caller proofs and scoped symbolic answers stay separate.
pub(crate) struct Cleanup {
    needed: NeedsDrop,
}
impl Cleanup {
    pub(crate) fn new(facts: &Declarations) -> Self {
        Self {
            needed: NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item)),
        }
    }
    #[cfg(test)]
    pub(crate) const fn steps(&self) -> usize {
        self.needed.steps
    }
}
struct Context<'a> {
    premises: Premises,
    cleanup: &'a mut Cleanup,
}

/// Materialize one selected body without resolving, parsing or inferring syntax.
/// Original source identities and control flow remain unchanged. Promotion values
/// are recomputed from the specialized DAG, never copied across argument contexts.
/// # Errors
/// Rejects foreign instances, incomplete substitutions and missing cleanup scopes.
pub fn body(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
) -> Result<(Body, drops::Requirements), &'static str> {
    let inputs = parameters(facts, instance)?;
    if inputs.comparison.is_some() {
        return Err("projected instance requires its lifetime contract");
    }
    let mut substitution = Bound::parameters(&facts.types, &inputs.bindings)?;
    let mut cleanup = Cleanup::new(facts);
    materialize(
        facts,
        template,
        instance,
        &mut substitution,
        None,
        None,
        Context {
            premises: Premises::Required,
            cleanup: &mut cleanup,
        },
    )
}

pub struct Materialized {
    pub body: Body,
    pub destruction: drops::Requirements,
    pub contract: super::regions::Contract,
}
/// Materialize a body and its contract in a fresh instance proof context.
/// Caller lifetimes contained in type arguments become distinct local universals;
/// original template variables and nested callable binders keep their identities.
/// # Errors
/// Rejects inconsistent templates, invalid substitutions and exhausted resources.
pub fn with_contract(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
    original: &super::regions::Contract,
) -> Result<Materialized, &'static str> {
    let mut cleanup = Cleanup::new(facts);
    with_premises(
        facts,
        template,
        instance,
        original,
        Context {
            premises: Premises::Required,
            cleanup: &mut cleanup,
        },
    )
}

/// Only demand from a fully proved caller may inherit its checked callee premises.
/// Unsealed/manual instances continue to require every implementation lifetime debt.
pub(crate) fn from_checked_caller(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
    original: &super::regions::Contract,
    cleanup: &mut Cleanup,
) -> Result<Materialized, &'static str> {
    with_premises(
        facts,
        template,
        instance,
        original,
        Context {
            premises: Premises::Declared,
            cleanup,
        },
    )
}

fn with_premises(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
    original: &super::regions::Contract,
    context: Context<'_>,
) -> Result<Materialized, &'static str> {
    if original.source != template.body.source {
        return Err("instance contract belongs to another template");
    }
    let mut inputs = parameters(facts, instance)?;
    let (mut substitution, mut contract) = contracts::specialize(facts, original, &mut inputs)?;
    let (body, destruction) = materialize(
        facts,
        template,
        instance,
        &mut substitution,
        Some(&mut contract),
        inputs.comparison,
        context,
    )?;
    Ok(Materialized {
        body,
        destruction,
        contract,
    })
}

fn materialize(
    facts: &mut Declarations,
    template: &Template,
    instance: Instance,
    substitution: &mut Bound,
    contract: Option<&mut super::regions::Contract>,
    comparison: Option<(crate::types::TypeId, crate::types::TypeId)>,
    context: Context<'_>,
) -> Result<(Body, drops::Requirements), &'static str> {
    let Context { premises, cleanup } = context;
    let source = &template.body;
    let Owner::Function { definition, .. } = source.owner else {
        return Err("instance template is not a function");
    };
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("missing callable instance");
    };
    if item.0 != definition.0 || source.instance.is_some() {
        return Err("instance does not select an original function template");
    }
    if facts.types.contains_parameters(instance.ty()) != Some(false) {
        return Err("executable instance has unresolved type or const parameters");
    }
    let mut body = Body {
        instance: Some(instance),
        owner: Owner::Function {
            definition,
            signature,
        },
        source: source.source,
        result: source.result,
        owned: None,
        storage: source.storage.clone(),
        promotions: None,
        resolved_callables: None,
        locals: source.locals.clone(),
        values: source.values.clone(),
        projections: source.projections.clone(),
        temporaries: source.temporaries.clone(),
        guards: source.guards.clone(),
        guard_places: source.guard_places.clone(),
        scalars: source.scalars.clone(),
        arguments: source.arguments.clone(),
        statements: source.statements.clone(),
        blocks: source.blocks.clone(),
    };
    let projected = substitute(facts, &mut body, substitution)?;
    if projected || comparison.is_some() || projections::needs_field_normalization(facts, &body)? {
        let contract = contract.ok_or("projected instance requires its lifetime contract")?;
        if matches!(premises, Premises::Declared)
            && let Some(assumptions) = crate::sema::callee_premises::specialize(
                facts,
                definition,
                substitution,
                contract.universals,
            )?
        {
            contract.universals = assumptions.universals;
            contract.assumptions.extend(assumptions.regions);
            contract.type_assumptions.extend(assumptions.types);
        }
        projections::normalize(facts, &mut body, contract, comparison)?;
    }
    let needed = &mut cleanup.needed;
    if let Some(owned) = &source.owned {
        let mut concrete = Box::<drops::values::Facts>::default();
        for &temporary in &owned.created {
            let ty = body
                .values
                .get(temporary.value.0 as usize)
                .ok_or("invalid template owned value")?
                .ty;
            if needed.demand(&facts.types, &facts.aggregates, ty)? {
                concrete.insert(temporary)?;
            }
        }
        for &returned in &owned.returned {
            if concrete.creation(returned.value).is_some() {
                concrete.returned.push(returned);
            }
        }
        if !concrete.created.is_empty() {
            body.owned = Some(concrete);
        }
    }
    let requirements = drops::requirements(facts, &body, needed)?;
    Ok((body, requirements))
}

type Binding = (crate::types::parameters::ParameterId, Argument);
struct Inputs {
    instance: Instance,
    bindings: Vec<Binding>,
    comparison: Option<(crate::types::TypeId, crate::types::TypeId)>,
}
fn parameters(facts: &mut Declarations, instance: Instance) -> Result<Inputs, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("invalid callable instance");
    };
    let callable = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("missing callable template")?;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("missing callable parameters")?;
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid callable arguments")?
        .collect::<Vec<_>>();
    let early = callable.parameters as usize;
    if arguments.len() != early.saturating_add(parameters.len()) {
        return Err("callable instance argument count differs from its template");
    }
    let mut regions = Vec::with_capacity(early);
    for argument in &arguments[..early] {
        let Argument::Lifetime(region) = *argument else {
            return Err("callable early argument is not a lifetime");
        };
        regions.push(region);
    }
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments[early..].iter().copied())
        .collect::<Vec<_>>();
    let mut selected = Bound::parameters_captured(&facts.types, &bindings, regions)?;
    let expected = selected.apply(&mut facts.types, callable.signature)?;
    let projected = facts.types.contains_projections(expected) == Some(true);
    if !projected && expected != signature {
        return Err("instance signature differs from substituted template");
    }
    Ok(Inputs {
        instance,
        bindings,
        comparison: projected.then_some((expected, signature)),
    })
}

fn substitute(
    facts: &mut Declarations,
    body: &mut Body,
    bound: &mut Bound,
) -> Result<bool, &'static str> {
    let mut projected = false;
    let mut ty = |original| {
        let result = bound.apply(&mut facts.types, original)?;
        if facts.types.contains_parameters(result) != Some(false) {
            return Err("instance body retains an unresolved type or const parameter");
        }
        if facts.types.contains_projections(result) == Some(true) {
            projected = true;
        } else {
            facts.aggregates.prepare(&mut facts.types, result)?;
        }
        Ok(result)
    };
    body.result = ty(body.result)?;
    for local in &mut body.locals {
        local.ty = ty(local.ty)?;
    }
    for value in &mut body.values {
        value.ty = ty(value.ty)?;
    }
    for projection in &mut body.projections {
        projection.ty = ty(projection.ty)?;
    }
    for value in &mut body.values {
        match &mut value.kind {
            Rvalue::Call { signature, .. } => {
                *signature = bound.apply(&mut facts.types, *signature)?;
                projected |= facts.types.contains_projections(*signature) == Some(true);
            }
            Rvalue::AssociatedConstant(instance) => {
                *instance = instance.substitute(&mut facts.types, bound)?;
            }
            Rvalue::ConstArgument(id) => {
                let Argument::Const(actual) =
                    bound.apply_argument(&mut facts.types, Argument::Const(*id))?
                else {
                    return Err("const operand substitution changed argument kind");
                };
                if facts.types.constant(actual).is_none() {
                    return Err("instance operand retains an unresolved const parameter");
                }
                *id = actual;
            }
            _ => {}
        }
    }
    for projection in &mut body.projections {
        if let ProjectionKind::IndexParameter { index, length } = projection.kind {
            let length = Length::Parameter(length)
                .substitute(&mut facts.types, bound)?
                .known()
                .ok_or("instance index retains an unresolved array bound")?;
            projection.kind = ProjectionKind::Index { index, length };
        }
        if let ProjectionKind::Dereference { region, .. } = &mut projection.kind {
            let Argument::Lifetime(actual) =
                bound.apply_argument(&mut facts.types, Argument::Lifetime(*region))?
            else {
                return Err("dereference substitution changed argument kind");
            };
            *region = actual;
        }
    }
    Ok(projected)
}
