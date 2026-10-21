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
    fn context(self) -> Context {
        match self {
            Self::Callable(instance) => Context::Callable(instance),
            Self::Anonymous(instance) => instance.enclosing(),
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
    };
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
    let (mut substitution, mut contract) = contracts::specialize(facts, original, &mut inputs)?;
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

fn initializer_inputs(
    facts: &Declarations,
    instance: crate::sema::values::Instance,
) -> Result<Inputs, &'static str> {
    let selected = instance.selected_arguments(facts)?;
    Ok(Inputs {
        instance: None,
        bindings: selected.bindings,
        captures: selected.regions,
        comparison: None,
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
