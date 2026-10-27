//! Substitute retained typed CFGs; the result still requires an instance proof.
mod allocated_owners;
mod anonymous;
mod caller_transport;
mod initializers;
mod parent_results;
mod rigid_values;
pub(crate) use parent_results::prepare as source_inline_results;
#[cfg(test)]
#[path = "../../tests/invariants/seals.rs"]
mod seals;
pub(crate) use initializers::{
    source as source_initializer, with_contract as selected_initializer,
};
mod closures;
mod contracts;
pub(crate) use closures::{
    ProofCursor, execution as closure_execution, retain_proof as retain_closure_proof,
};
pub(crate) use closures::{
    normalize as normalize_closure_execution, self_type as closure_self_type,
    substitute as substitute_closure_execution,
};
mod inputs;
mod materialized_copy;
pub(crate) use anonymous::{
    close_operands as close_inline_operands, conditional_result as conditional_anonymous_result,
    original_context as original_inline_context, with_result as anonymous_result,
};
use inputs::Inputs;
mod projections;
use super::{
    drops,
    ir::{Body, ProjectionKind, Rvalue},
    templates::Template,
};
use crate::{
    sema::model::Declarations,
    types::{
        arguments::Argument, destruction::NeedsDrop, instance::Instance, length::Length,
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
    let inputs = inputs::select(facts, instance)?;
    if inputs.comparison.is_some() {
        return Err("projected instance requires its lifetime contract");
    }
    if !inputs.captures.is_empty() || inputs.connections.is_some() {
        return Err("inherited closure lifetimes require their checked contract");
    }
    let mut substitution = Bound::parameters(&facts.types, &inputs.bindings)?;
    let mut cleanup = Cleanup::new(facts);
    materialize(
        facts,
        template,
        anonymous::Selection::Callable(instance),
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
    copy_scope: Option<Box<super::copy_requirements::materialized::InstanceScope>>,
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
    if original.key != template.body.key() || original.source != template.body.source {
        return Err("instance contract belongs to another template");
    }
    if let super::ir::Owner::Function {
        signature,
        execution,
        ..
    } = template.body.owner
        && (execution.is_some() || original.signature != Some(signature))
    {
        return Err("instance original signature differs from its exact template");
    }
    let original_scope =
        super::copy_requirements::materialized::capture(facts, &template.body, original, instance)?;
    let mut inputs = inputs::select(facts, instance)?;
    let (mut substitution, mut contract) =
        contracts::specialize(facts, original, &mut inputs, context.premises)?;
    let (body, destruction) = materialize(
        facts,
        template,
        anonymous::Selection::Callable(instance),
        &mut substitution,
        Some(&mut contract),
        inputs.comparison,
        context,
    )?;
    let copy_scope = original_scope
        .map(|scope| scope.finish(facts, &body, &mut contract))
        .transpose()?;
    Ok(Materialized {
        body,
        destruction,
        contract,
        copy_scope,
    })
}

fn assembly_instance_work(facts: &mut Declarations, source: &Body) -> Result<(), &'static str> {
    if let Some(data) = source.assembly.as_deref() {
        let steps = data
            .instructions
            .len()
            .saturating_add(data.templates.len())
            .saturating_add(data.pieces.len())
            .saturating_add(data.operands.len());
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
        if size_of::<super::assembly::Payload>().saturating_add(data.allocation_bytes())
            > 64_000_000
        {
            return Err("assembly instance payload exceeds existing generic body storage budget");
        }
    }

    Ok(())
}

fn materialize(
    facts: &mut Declarations,
    template: &Template,
    selection: anonymous::Selection,
    substitution: &mut Bound,
    mut contract: Option<&mut super::regions::Contract>,
    comparison: Option<(crate::types::TypeId, crate::types::TypeId)>,
    context: Context<'_>,
) -> Result<(Body, drops::Requirements), &'static str> {
    let Context { premises, cleanup } = context;
    let source = &template.body;
    assembly_instance_work(facts, source)?;

    let (owner, definition, instance) = selection.owner(facts, source)?;
    if instance
        .is_some_and(|instance| facts.types.contains_parameters(instance.ty()) != Some(false))
    {
        return Err("executable instance has unresolved type or const parameters");
    }
    let mut body = Body {
        assembly: source.assembly.clone(),
        mutable_statics: source.mutable_statics.clone(),
        static_types: source.static_types.clone(),
        const_context: source.const_context,
        instance,
        owner,
        source: source.source,
        result: source.result,
        owned: None,
        storage: source.storage.clone(),
        promotions: None,
        resolved_callables: source.unproved_callable_inputs(facts)?,
        closure_calls: None,
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
    caller_transport::prepare_body(facts, source, &mut body, selection.caller(facts)?)?;
    let projected = substitute(facts, &mut body, substitution)?;
    if matches!(premises, Premises::Declared)
        && let Some(contract) = contract.as_deref_mut()
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
    anonymous::operands(facts, &mut body, selection)?;
    if projected
        || comparison.is_some()
        || contract
            .as_deref()
            .map(|contract| projections::needs_clause_normalization(facts, contract))
            .transpose()?
            .unwrap_or(false)
        || projections::needs_field_normalization(facts, &body)?
    {
        let contract = contract
            .as_deref_mut()
            .ok_or("projected instance requires its lifetime contract")?;
        caller_transport::normalize(facts, &mut body, contract, comparison)?;
    }
    inputs::finish_owner(facts, &mut body.owner, contract.as_deref())?;
    facts.prepare_opaque_storage()?;
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
    if let Some(data) = &mut body.assembly {
        for operand in &mut data.operands {
            operand.ty = operand.ty.map(&mut ty).transpose()?;
        }
    }
    for projection in &mut body.projections {
        projection.ty = ty(projection.ty)?;
    }
    for value in &mut body.values {
        match &mut value.kind {
            Rvalue::Intrinsic { instance, .. } => {
                let ty = bound.apply(&mut facts.types, instance.ty())?;
                projected |= facts.types.contains_projections(ty) == Some(true);
                *instance = crate::types::instance::Instance::new(&facts.types, ty)?;
            }
            Rvalue::Unsize { conversion, .. } => {
                *conversion = conversion.try_map(|ty| {
                    let ty = bound.apply(&mut facts.types, ty)?;
                    projected |= facts.types.contains_projections(ty) == Some(true);
                    Ok::<_, &'static str>(ty)
                })?;
            }
            Rvalue::Call { signature, .. } | Rvalue::Invoke { signature, .. } => {
                if !matches!(
                    facts.types.kind(*signature),
                    Some(crate::types::Kind::Function {
                        bound_regions: 0,
                        ..
                    })
                ) {
                    return Err("instance call signature retains unopened lifetimes");
                }
                // These roots already live in the body. The empty call record
                // introduces no new scope around substituted argument lifetimes.
                *signature = bound.signature(&mut facts.types, *signature, 0)?;
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
    body.specialize_caller_inputs(facts)?;
    Ok(projected)
}
