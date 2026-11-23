//! Source integer bit counts and permutations retain the selected width and signature.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    support::bindings::SourceIntrinsic,
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Effect {
    Ones,
    LeadingZeros,
    TrailingZeros,
    SwappedBytes,
    ReversedBits,
    RotateLeft,
    RotateRight,
}
impl Effect {
    pub(crate) const fn is_rotation(self) -> bool {
        matches!(self, Self::RotateLeft | Self::RotateRight)
    }
}
/// The `_nonzero` declarations are unsafe and leave a zero input undefined.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Operation {
    effect: Effect,
    nonzero: bool,
}
impl Operation {
    pub(crate) const fn is_rotation(self) -> bool {
        self.effect.is_rotation()
    }

    const fn arity(self) -> usize {
        if self.is_rotation() { 2 } else { 1 }
    }

    /// Counts return `u32`; permutations keep the input's type.
    const fn output(self, input: TypeId) -> TypeId {
        match self.effect {
            Effect::SwappedBytes
            | Effect::ReversedBits
            | Effect::RotateLeft
            | Effect::RotateRight => input,
            Effect::Ones | Effect::LeadingZeros | Effect::TrailingZeros => {
                TypeId::primitive(Primitive::U32)
            }
        }
    }
}

pub(crate) fn operation(kind: SourceIntrinsic) -> Option<Operation> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return None;
    };
    named(identity.name())
}

pub(crate) fn named(name: &str) -> Option<Operation> {
    let (effect, nonzero) = match name {
        "ctpop" => (Effect::Ones, false),
        "ctlz" => (Effect::LeadingZeros, false),
        "ctlz_nonzero" => (Effect::LeadingZeros, true),
        "cttz" => (Effect::TrailingZeros, false),
        "cttz_nonzero" => (Effect::TrailingZeros, true),
        "bswap" => (Effect::SwappedBytes, false),
        "bitreverse" => (Effect::ReversedBits, false),
        "rotate_left" => (Effect::RotateLeft, false),
        "rotate_right" => (Effect::RotateRight, false),
        _ => return None,
    };
    Some(Operation { effect, nonzero })
}

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
    input: TypeId,
    output: TypeId,
    width: u16,
    operation: Operation,
}
impl Selection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn input(&self) -> TypeId {
        self.input
    }
    pub(crate) const fn output(&self) -> TypeId {
        self.output
    }
    pub(crate) const fn effect(&self) -> Effect {
        self.operation.effect
    }
    pub(crate) const fn arity(&self) -> usize {
        self.operation.arity()
    }
    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        let inputs = [self.input, TypeId::primitive(Primitive::U32)];
        super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            &inputs[..self.arity()],
            self.output,
            self.operation.nonzero,
            Convention::RUST,
        )
    }
    /// Evaluate one selected integer input at its declared width. Rotations
    /// have a separate distance. A `_nonzero` declaration forbids zero.
    pub(crate) fn evaluate(&self, bits: u128) -> Option<u128> {
        let width = u32::from(self.width);
        let bits = bits & (u128::MAX >> (128 - width));
        if bits == 0 && self.operation.nonzero {
            return None;
        }
        Some(match self.operation.effect {
            Effect::Ones => u128::from(bits.count_ones()),
            Effect::LeadingZeros => u128::from(bits.leading_zeros() - (128 - width)),
            Effect::TrailingZeros => u128::from(bits.trailing_zeros().min(width)),
            Effect::SwappedBytes => bits.swap_bytes() >> (128 - width),
            Effect::ReversedBits => bits.reverse_bits() >> (128 - width),
            Effect::RotateLeft | Effect::RotateRight => return None,
        })
    }
}

impl Catalog {
    pub(crate) fn bit_count(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        let Some(operation) = operation(kind) else {
            return Ok(None);
        };
        if operation.is_rotation() && !self.uses_source_fallback(definition) {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source bit count selection work budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("source bit count provenance requires its wider transport profile");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source bit count requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source bit count completed declaration changed");
        }
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source bit count has invalid instance arguments")?;
        let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
            return Err("source bit count requires its complete one-type instance");
        };
        let Some((_, width)) = (match facts.types.kind(input) {
            Some(Kind::Primitive(primitive)) => primitive.integer(),
            _ => None,
        }) else {
            return Err("source bit count execution requires a closed integer type");
        };
        let output = operation.output(input);
        let parameters = [input, TypeId::primitive(Primitive::U32)];
        let inputs = &parameters[..operation.arity()];
        if !matches!(facts.types.kind(signature), Some(Kind::Function {
            bound_regions: 0, parameters, result, unsafe_, convention: Convention::RUST,
        }) if unsafe_ == operation.nonzero && result == output
            && facts.types.list(parameters) == Some(inputs))
        {
            return Err("source bit intrinsic changed its exact integer Rust signature");
        }
        let roots =
            super::intrinsic_contracts::item_roots(&facts.types, [input, output], |work| {
                crate::sema::header_attributes::charge_source_contract_query(facts, work)
                    .map_err(|_| "source bit count opening work exhausted")
            })?
            .ok_or("source bit count has no canonical caller roots")?;
        if roots != [input, output] {
            return Err("closed source bit count roots changed through their callable binder");
        }
        Ok(Some(Selection {
            definition,
            signature,
            input,
            output,
            width,
            operation,
        }))
    }
    pub(crate) fn require_bit_count_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &Selection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, kind, _)) = self.intrinsic_record(selected.definition)? else {
            return Err("source bit count lost its attributed declaration");
        };
        if operation(kind) != Some(selected.operation) {
            return Err("source bit count changed its intrinsic identity");
        }
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source bit count lost its current source header")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
            || !facts.is_const_function(selected.definition)
        {
            return Err("source bit count requires its actual completed const declaration");
        }
        Ok(())
    }
}
