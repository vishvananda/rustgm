//! Integer bit counts bind real source declarations, never unsafe/const permission.
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
pub(crate) enum Count {
    Ones,
    LeadingZeros,
    TrailingZeros,
}
/// The `_nonzero` declarations are unsafe and leave a zero input undefined.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Operation {
    count: Count,
    nonzero: bool,
}

pub(crate) fn operation(kind: SourceIntrinsic) -> Option<Operation> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return None;
    };
    let (count, nonzero) = match identity.name() {
        "ctpop" => (Count::Ones, false),
        "ctlz" => (Count::LeadingZeros, false),
        "ctlz_nonzero" => (Count::LeadingZeros, true),
        "cttz" => (Count::TrailingZeros, false),
        "cttz_nonzero" => (Count::TrailingZeros, true),
        _ => return None,
    };
    Some(Operation { count, nonzero })
}

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
    input: TypeId,
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
    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            &[self.input],
            TypeId::primitive(Primitive::U32),
            self.operation.nonzero,
            Convention::RUST,
        )
    }
    /// Count an exact integer value of the selected width. `None` is the
    /// undefined zero input of a `_nonzero` declaration.
    pub(crate) fn count(&self, bits: u128) -> Option<u32> {
        let width = u32::from(self.width);
        let bits = bits & (u128::MAX >> (128 - width));
        if bits == 0 && self.operation.nonzero {
            return None;
        }
        Some(match self.operation.count {
            Count::Ones => bits.count_ones(),
            Count::LeadingZeros => bits.leading_zeros() - (128 - width),
            Count::TrailingZeros => bits.trailing_zeros().min(width),
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
        let output = TypeId::primitive(Primitive::U32);
        if !matches!(facts.types.kind(signature), Some(Kind::Function {
            bound_regions: 0, parameters, result, unsafe_, convention: Convention::RUST,
        }) if unsafe_ == operation.nonzero && result == output
            && facts.types.list(parameters) == Some(&[input]))
        {
            return Err("source bit count changed its exact Rust (T) -> u32 signature");
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
