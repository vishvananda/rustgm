//! One paid formal binding index serves every owned invocation in a body.
use super::Work;
use crate::{
    body::ir::Body,
    resolve::model::DefId,
    sema::model::Declarations,
    support::hash::HashMap,
    types::{
        ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{ParameterId, ParameterKind},
    },
};

pub(super) struct Bindings {
    types: HashMap<ParameterId, Option<TypeId>>,
}
impl Bindings {
    pub(super) fn new(
        facts: &Declarations,
        body: &Body,
        owner: DefId,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        work.charge(1)?;
        let (template, formals) = facts
            .lifetimes
            .callables
            .registration(ItemId(owner.0))
            .ok_or("const invocation has no completed callable formal order")?;
        // No per-invocation scan of unrelated parameters and no retained Body
        // column. The exact full instance supplies inherited and own slots alike.
        work.charge(1 + formals.len() + template.parameters as usize)?;
        let mut actual = if let Some(instance) = body.instance {
            let Some(Kind::FunctionItem {
                item,
                arguments,
                signature,
            }) = facts.types.kind(instance.ty())
            else {
                return Err("const invocation lost its complete function instance");
            };
            if item.0 != owner.0
                || body.owner.signatures().map(|(public, _)| public) != Some(signature)
            {
                return Err("const invocation belongs to another full callable instance");
            }
            let mut arguments = facts
                .types
                .arguments(arguments)
                .ok_or("invalid const invocation instance arguments")?;
            if arguments.len() != template.parameters as usize + formals.len() {
                return Err("const invocation changed its complete generic arity");
            }
            for _ in 0..template.parameters {
                if !matches!(arguments.next(), Some(Argument::Lifetime(_))) {
                    return Err("const invocation changed an early lifetime argument");
                }
            }
            Some(arguments)
        } else {
            None
        };
        let mut types = HashMap::default();
        types
            .try_reserve(formals.len())
            .map_err(|_| "const invocation binding allocation failed")?;
        for &id in formals {
            let parameter = facts
                .types
                .parameter(id)
                .ok_or("const invocation has invalid callable formal")?;
            let argument = actual.as_mut().map(Iterator::next);
            match (parameter.kind, argument) {
                (ParameterKind::Type { .. }, None | Some(Some(Argument::Type(_)))) => {
                    let ty = match argument {
                        Some(Some(Argument::Type(ty))) => Some(ty),
                        _ => None,
                    };
                    if types.insert(id, ty).is_some() {
                        return Err("const invocation repeats a callable formal");
                    }
                }
                (ParameterKind::Const { .. }, None | Some(Some(Argument::Const(_)))) => {}
                _ => return Err("const invocation changed a generic argument kind"),
            }
        }
        Ok(Self { types })
    }

    pub(super) fn receiver(
        &self,
        facts: &Declarations,
        original: TypeId,
        signature: TypeId,
        work: &mut Work,
    ) -> Result<TypeId, &'static str> {
        work.charge(2)?;
        let Some(Kind::Parameter(id)) = facts.types.kind(original) else {
            return Err("const invocation requires a direct generic receiver parameter");
        };
        let selected = self
            .types
            .get(&id)
            .ok_or("const invocation receiver is outside its callable formals")?;
        let Some(actual) = *selected else {
            return Ok(original);
        };
        work.charge(4)?;
        let public = match facts.types.kind(actual) {
            Some(Kind::FunctionItem { signature, .. }) => signature,
            Some(Kind::Function { .. }) => actual,
            _ => return Err("const invocation requires a complete function or pointer receiver"),
        };
        // The original typed call is checked independently before materialization;
        // instance construction and borrow proof retain the complete substitution.
        // Here the actual bound receiver must agree with every current operand.
        if public != signature
            || facts.types.needs_region_context(actual) != Some(false)
            || facts.types.contains_parameters(actual) != Some(false)
            || facts.types.contains_projections(actual) != Some(false)
        {
            return Err("const invocation changed its complete receiver signature");
        }
        Ok(actual)
    }
}
