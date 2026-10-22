//! Anonymous bodies substitute exact enclosing bindings without acquiring callable identity.
use super::{
    Cleanup, Context as ProofContext, Declarations, Inputs, Materialized, Premises, contracts,
    inputs,
};
use crate::{
    body::{
        ir::{Body, BodyKey, Owner, Rvalue},
        regions::Contract,
        templates::Template,
    },
    resolve::model::DefId,
    sema::inline_constants::{Context, InlineInstance},
    types::instance::Instance,
};

#[derive(Clone, Copy)]
pub(super) enum Selection {
    Callable(Instance),
    Anonymous(InlineInstance),
    Initializer(crate::sema::values::Instance),
    SourceInitializer(crate::sema::values::execution::SourceComputation),
}
impl Selection {
    pub(super) fn owner(
        self,
        facts: &Declarations,
        source: &Body,
    ) -> Result<(Owner, DefId, Option<Instance>), &'static str> {
        match self {
            Self::Callable(instance) => {
                let (owner, definition) = inputs::owner(facts, source, instance)?;
                Ok((owner, definition, Some(instance)))
            }
            Self::SourceInitializer(witness) => {
                let (instance, original) = witness.identity(facts)?;
                if original != source.source {
                    return Err("source initializer selects another anonymous operand owner");
                }
                Self::Initializer(instance).owner(facts, source)
            }
            Self::Initializer(instance) => {
                let original = facts
                    .initializer_index(instance.definition)
                    .and_then(|index| facts.initializers.get(index))
                    .ok_or("selected initializer has no original declaration")?;
                if source.instance.is_some()
                    || source.source != original.semantic_source()
                    || !matches!(source.owner, Owner::Initializer { definition, kind: crate::sema::model::InitializerKind::Const } if definition==instance.definition)
                {
                    return Err("selected initializer selects another original body");
                }
                Ok((
                    Owner::InitializerInstance {
                        instance,
                        kind: original.kind,
                    },
                    instance.definition,
                    None,
                ))
            }
            Self::Anonymous(instance) => {
                let selected = InlineInstance::select(facts, source.source, instance.enclosing())?;
                if selected != instance
                    || source.instance.is_some()
                    || !matches!(source.owner, Owner::AnonymousConstant)
                {
                    return Err("anonymous instance selects another original body");
                }
                let definition = facts
                    .inline_constants
                    .as_deref()
                    .and_then(|r| r.get(source.source))
                    .and_then(|b| b.declaration)
                    .ok_or("anonymous template has no declaration")?;
                Ok((Owner::AnonymousInstance(instance), definition, None))
            }
        }
    }
    pub(super) fn caller(
        self,
        facts: &Declarations,
    ) -> Result<crate::sema::values::execution::CallerInstance, &'static str> {
        use crate::sema::values::execution::CallerInstance;
        Ok(match self {
            Self::Callable(instance) => CallerInstance::Callable(instance),
            Self::Anonymous(instance) => CallerInstance::Inline(instance),
            Self::Initializer(instance) => CallerInstance::Initializer(instance),
            Self::SourceInitializer(witness) => {
                CallerInstance::Initializer(witness.identity(facts)?.0)
            }
        })
    }
    fn context(self) -> Context {
        match self {
            Self::Callable(instance) => Context::Callable(instance),
            Self::Anonymous(instance) => instance.enclosing(),
            Self::Initializer(instance) => Context::Initializer(instance),
            Self::SourceInitializer(witness) => Context::SourceInitializer(witness),
        }
    }
}
pub(super) fn operands(
    facts: &Declarations,
    body: &mut Body,
    selection: Selection,
) -> Result<(), &'static str> {
    if facts.inline_constants.is_none() {
        return Ok(());
    }
    for value in &mut body.values {
        if let Rvalue::InlineInstance(instance) = value.kind {
            let selected = InlineInstance::select(facts, instance.source(), selection.context())?;
            if selected != instance {
                return Err("copied inline operand belongs to another enclosing context");
            }
        }
        if let Rvalue::InlineConstant(source) = value.kind {
            let row = facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(source))
                .ok_or("inline operand lost its typed owner")?;
            if row.evaluation == crate::sema::inline_constants::Evaluation::DeferredTemplate {
                value.kind = Rvalue::InlineInstance(InlineInstance::select(
                    facts,
                    source,
                    selection.context(),
                )?);
            }
        }
    }
    Ok(())
}
pub(crate) fn with_contract(
    facts: &mut Declarations,
    template: &Template,
    instance: InlineInstance,
    original: &Contract,
) -> Result<Materialized, &'static str> {
    if original.key != template.body.key()
        || original.source != instance.source()
        || original.output.is_some()
    {
        return Err("anonymous specialized output interface is not implemented");
    }
    let mut inputs = anonymous_inputs(facts, instance, original)?;
    let (substitution, contract) =
        contracts::specialize(facts, original, &mut inputs, super::Premises::Required)?;
    materialize_anonymous(facts, template, instance, substitution, contract)
}

pub(crate) fn with_result(
    facts: &mut Declarations,
    template: &Template,
    instance: InlineInstance,
    requested: crate::types::TypeId,
    original: &Contract,
) -> Result<super::initializers::SourceMaterialized, &'static str> {
    if !matches!(instance.enclosing(), Context::SourceInitializer(_))
        || (original.output.is_none()
            && facts.types.needs_region_context(template.body.result) == Some(false))
    {
        return with_contract(facts, template, instance, original)
            .map(super::initializers::SourceMaterialized::plain);
    }
    if original.key != template.body.key() || original.source != instance.source() {
        return Err("anonymous result selects another original contract");
    }
    let mut inputs = anonymous_inputs(facts, instance, original)?;
    let (mut substitution, mut contract, imported) =
        contracts::source_result(facts, original, &mut inputs, requested)?;
    let actual = super::caller_transport::result_root(
        facts,
        template.body.source,
        crate::sema::values::execution::CallerInstance::Inline(instance),
        template.body.result,
    )?;
    let before = substitution.steps;
    let actual = substitution.apply(&mut facts.types, actual)?;
    facts.body_work.canonical_steps += substitution.steps - before;
    let obligation = contract.obligations.len();
    contract.obligations.push(crate::body::regions::Obligation {
        source: original.source,
        kind: crate::body::regions::Kind::Coercion {
            actual,
            expected: imported,
            unsize: false,
            reference: None,
        },
    });
    let materialized = materialize_anonymous(facts, template, instance, substitution, contract)?;
    let expected = match materialized
        .contract
        .obligations
        .get(obligation)
        .map(|row| &row.kind)
    {
        Some(crate::body::regions::Kind::Coercion {
            actual,
            expected,
            unsize: false,
            reference: None,
        }) if *actual == materialized.body.result => *expected,
        _ => return Err("anonymous result lost its actual coercion obligation"),
    };
    result_shape(facts, materialized.body.result, expected)?;
    Ok(super::initializers::SourceMaterialized::interface(
        materialized,
        requested,
        expected,
        obligation,
    ))
}

fn result_shape(
    facts: &mut Declarations,
    actual: crate::types::TypeId,
    expected: crate::types::TypeId,
) -> Result<(), &'static str> {
    let mut shape = crate::types::substitute::Bound::shape();
    let result = (|| {
        let actual = shape.apply_bounded(&mut facts.types, actual, 1_000_000, 64 * 1024 * 1024)?;
        let expected = shape.apply_bounded(
            &mut facts.types,
            expected,
            1_000_000_usize.saturating_sub(shape.steps),
            64 * 1024 * 1024,
        )?;
        if actual != expected {
            return Err("anonymous result requires a non-lifetime representation conversion");
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(shape.steps);
    result
}

fn anonymous_inputs(
    facts: &mut Declarations,
    instance: InlineInstance,
    original: &Contract,
) -> Result<Inputs, &'static str> {
    let mut inputs = match instance.enclosing() {
        Context::Callable(callable) => {
            if matches!(
                facts.types.kind(callable.ty()),
                Some(crate::types::Kind::Closure { .. })
            ) {
                inputs::closure(facts, callable, true)?
            } else {
                inputs::select(facts, callable)?
            }
        }
        Context::Initializer(initializer) => initializer_inputs(facts, initializer)?,
        Context::SourceInitializer(witness) => {
            let result = witness.result(facts)?;
            let selected = witness.resolve(facts, result)?;
            initializer_inputs(facts, selected.selected)?
        }
    };
    inputs.caller = crate::sema::values::execution::CallerInstance::Inline(instance);
    // A zero-universal anonymous contract cannot mention enclosing lifetime slots;
    // the full selected callable still validates and remains in its cache identity.
    if original.universals == 0 {
        inputs.captures.clear();
    }
    if inputs.captures.len() > original.universals as usize {
        return Err("anonymous enclosing lifetime interface requires explicit slot transport");
    }
    // Anonymous bodies have no callable signature comparison or receiver execution.
    inputs.comparison = None;
    Ok(inputs)
}

fn materialize_anonymous(
    facts: &mut Declarations,
    template: &Template,
    instance: InlineInstance,
    mut substitution: crate::types::substitute::Bound,
    mut contract: Contract,
) -> Result<Materialized, &'static str> {
    contract.key = BodyKey::AnonymousInstance(instance);
    let mut cleanup = Cleanup::new(facts);
    let (body, destruction) = super::materialize(
        facts,
        template,
        Selection::Anonymous(instance),
        &mut substitution,
        Some(&mut contract),
        None,
        ProofContext {
            premises: Premises::Required,
            cleanup: &mut cleanup,
        },
    )?;
    Ok(Materialized {
        body,
        destruction,
        contract,
    })
}

pub(super) fn initializer_inputs(
    facts: &Declarations,
    instance: crate::sema::values::Instance,
) -> Result<Inputs, &'static str> {
    let selected = instance.selected_arguments(facts)?;
    Ok(Inputs {
        caller: crate::sema::values::execution::CallerInstance::Initializer(instance),
        instance: None,
        bindings: selected.bindings,
        captures: selected.regions,
        comparison: None,
        connections: None,
    })
}

/// Lifetime-only source bodies do not pass through generic materialization.
/// Select their exact callable identity without changing the body owner or ABI.
pub(crate) fn close_operands(facts: &Declarations, body: &mut Body) -> Result<(), &'static str> {
    if !facts
        .inline_constants
        .as_deref()
        .is_some_and(|registry| registry.has_children(body.source))
    {
        return Ok(());
    }
    let owner = match body.owner {
        Owner::Function { definition, .. } => Some(definition),
        Owner::Closure { identity, .. } => facts
            .closures
            .identity(identity)
            .and_then(|closure| closure.owner),
        _ => None,
    };
    if owner.is_some_and(|owner| facts.generics.has_parameters(owner)) {
        return Ok(());
    }
    let ty = match body.owner {
        Owner::Function { definition, .. } => facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten(),
        Owner::Closure { .. } => facts.node(body.source).and_then(|node| node.ty),
        _ => None,
    };
    let Some(ty) = ty else {
        return Ok(());
    };
    if facts.types.contains_parameters(ty) != Some(false)
        || facts.types.contains_projections(ty) != Some(false)
    {
        return Ok(());
    }
    let callable = if matches!(
        facts.types.kind(ty),
        Some(crate::types::Kind::Closure { .. })
    ) {
        Instance::closure(&facts.types, ty)?
    } else {
        Instance::new(&facts.types, ty)?
    };
    operands(facts, body, Selection::Callable(callable))
}
