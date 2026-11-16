//! The portable SIMD intrinsics bind their real source declarations; an
//! instance carries the closed types its call passes and returns. Which
//! vector shapes an operation accepts is decided where it is lowered or
//! evaluated, as rustc decides it at monomorphization.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, model::DefId},
    sema::model::Declarations,
    support::bindings::SourceIntrinsic,
    types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};

/// The SIMD intrinsics that the 16-byte lowering implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SimdOperation {
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Bitmask,
    ReduceAll,
    Shuffle,
    Splat,
}
impl SimdOperation {
    pub(crate) fn of(kind: SourceIntrinsic) -> Option<Self> {
        let SourceIntrinsic::Declaration(identity) = kind else {
            return None;
        };
        Some(match identity.name() {
            "simd_eq" => Self::Eq,
            "simd_gt" => Self::Gt,
            "simd_lt" => Self::Lt,
            "simd_and" => Self::And,
            "simd_or" => Self::Or,
            "simd_bitmask" => Self::Bitmask,
            "simd_reduce_all" => Self::ReduceAll,
            "simd_shuffle" => Self::Shuffle,
            "simd_splat" => Self::Splat,
            _ => return None,
        })
    }
    /// The number of values the declaration takes.
    const fn arity(self) -> usize {
        match self {
            Self::Bitmask | Self::ReduceAll | Self::Splat => 1,
            Self::Eq | Self::Gt | Self::Lt | Self::And | Self::Or => 2,
            Self::Shuffle => 3,
        }
    }
}

/// One closed instance of a validated SIMD declaration.
#[derive(Clone, Copy)]
pub(crate) struct SimdSelection {
    pub definition: DefId,
    pub operation: SimdOperation,
    inputs: [TypeId; 3],
    pub output: TypeId,
}
impl SimdSelection {
    /// The value types of the call, in source order.
    pub(crate) fn inputs(&self) -> &[TypeId] {
        &self.inputs[..self.operation.arity()]
    }
    /// Whether a body call's opened signature is this instance's.
    pub(crate) fn opened_signature_matches(
        &self,
        store: &crate::types::Store,
        signature: TypeId,
    ) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            store,
            signature,
            self.inputs(),
            self.output,
            true,
            Convention::RUST,
        )
    }
}

impl Catalog {
    /// The closed instance of a reached SIMD declaration, or `None` for any
    /// other callable.
    pub(crate) fn simd_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SimdSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        let Some(operation) = SimdOperation::of(kind) else {
            return Ok(None);
        };
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source SIMD selection work budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("source SIMD intrinsic provenance requires its wider transport profile");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source SIMD intrinsic requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
            || function.body.is_some()
            || !facts.is_const_function(definition)
        {
            return Err("source SIMD intrinsic requires its own bodyless const declaration");
        }
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("source SIMD intrinsic changed its unsafe Rust signature");
        };
        let parameters = facts
            .types
            .list(parameters)
            .ok_or("source SIMD intrinsic has invalid inputs")?;
        if parameters.len() != operation.arity() {
            return Err("source SIMD intrinsic changed its input count");
        }
        let mut roots = [TypeId::UNIT; 4];
        roots[..parameters.len()].copy_from_slice(parameters);
        roots[3] = result;
        let opened = super::intrinsic_contracts::item_roots(&facts.types, roots, |work| {
            crate::sema::header_attributes::charge_source_contract_query(facts, work)
                .map_err(|_| "source SIMD intrinsic opening work exhausted")
        })?
        .ok_or("source SIMD intrinsic has no canonical caller roots")?;
        if opened != roots
            || roots
                .iter()
                .any(|&ty| facts.types.contains_parameters(ty) != Some(false))
        {
            return Err("source SIMD intrinsic execution requires closed types");
        }
        let mut inputs = [TypeId::UNIT; 3];
        inputs[..parameters.len()].copy_from_slice(parameters);
        if operation == SimdOperation::ReduceAll && result != TypeId::primitive(Primitive::Bool) {
            return Err("source simd_reduce_all changed its bool result");
        }
        Ok(Some(SimdSelection {
            definition,
            operation,
            inputs,
            output: result,
        }))
    }
}
