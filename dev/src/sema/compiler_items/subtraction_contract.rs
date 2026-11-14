//! Integer arithmetic binds real source declarations, never unsafe/const permission.
use super::Catalog;
use crate::lex::token::Punctuation;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::{LangItem, SourceIntrinsic},
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

pub(crate) fn wrapping_operator(kind: SourceIntrinsic) -> Option<Punctuation> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return None;
    };
    match identity.name() {
        "wrapping_add" => Some(Punctuation::Plus),
        "wrapping_sub" => Some(Punctuation::Minus),
        "wrapping_mul" => Some(Punctuation::Star),
        _ => None,
    }
}

pub(crate) fn unchecked_operator(kind: SourceIntrinsic) -> Option<Punctuation> {
    match kind {
        SourceIntrinsic::UncheckedSub => Some(Punctuation::Minus),
        SourceIntrinsic::Declaration(identity) => match identity.name() {
            "unchecked_add" => Some(Punctuation::Plus),
            "unchecked_mul" => Some(Punctuation::Star),
            "unchecked_div" => Some(Punctuation::Slash),
            "unchecked_rem" => Some(Punctuation::Percent),
            _ => None,
        },
        _ => None,
    }
}
pub(crate) fn unchecked_shift(kind: SourceIntrinsic) -> Option<Punctuation> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return None;
    };
    match identity.name() {
        "unchecked_shl" => Some(Punctuation::ShiftLeft),
        "unchecked_shr" => Some(Punctuation::ShiftRight),
        _ => None,
    }
}
pub(crate) fn scalar_arithmetic(kind: SourceIntrinsic) -> bool {
    unchecked_operator(kind).is_some()
        || wrapping_operator(kind).is_some()
        || overflowing_operator(kind).is_some()
        || unchecked_shift(kind).is_some()
}
pub(crate) fn overflowing_operator(kind: SourceIntrinsic) -> Option<Punctuation> {
    match kind {
        SourceIntrinsic::SubWithOverflow => Some(Punctuation::Minus),
        SourceIntrinsic::Declaration(identity) if identity.name() == "mul_with_overflow" => {
            Some(Punctuation::Star)
        }
        _ => None,
    }
}

pub(crate) struct SubtractionSelection {
    pub definition: DefId,
    pub signature: TypeId,
    pub input: TypeId,
    pub rhs: TypeId,
    pub output: TypeId,
    pub kind: SourceIntrinsic,
}
impl SubtractionSelection {
    pub(crate) fn opened_signature_matches(
        &self,
        store: &crate::types::Store,
        signature: TypeId,
    ) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            store,
            signature,
            &[self.input, self.rhs],
            self.output,
            unchecked_operator(self.kind).is_some() || unchecked_shift(self.kind).is_some(),
            Convention::RUST,
        )
    }
}
pub(super) fn validate(
    facts: &Declarations,
    function: Function,
    kind: SourceIntrinsic,
) -> Result<(), &'static str> {
    let arity = if unchecked_shift(kind).is_some() {
        2
    } else {
        1
    };
    let parameters = facts
        .generics
        .parameters
        .get(&function.definition)
        .map(Vec::as_slice)
        .ok_or("source arithmetic requires its own Copy type formals")?;
    if parameters.len() != arity {
        return Err("source arithmetic has the wrong own Copy type formal count");
    }
    let lifetime = facts
        .lifetimes
        .functions
        .get(&function.definition)
        .ok_or("source subtraction requires its completed lifetime scheme")?;
    if function.body.is_some()
        || !facts.is_const_function(function.definition)
        || lifetime.total != 0
        || lifetime.captures != 0
        || !lifetime.early.is_empty()
        || !lifetime.late.is_empty()
        || !lifetime.bounds.is_empty()
    {
        return Err(
            "source subtraction requires its own bodyless const declaration without regions",
        );
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source subtraction has no actual catalog")?;
    let mut inputs = [TypeId::UNIT; 2];
    for (index, &parameter) in parameters.iter().enumerate() {
        inputs[index] = formal(facts, function.definition, index, parameter)?;
    }
    if arity == 1 {
        inputs[1] = inputs[0];
        super::compare_intrinsic::copy_premise(facts, function.definition, inputs[0], catalog)?;
    } else {
        super::compare_intrinsic::copy_premises(facts, function.definition, inputs, catalog)?;
    }
    signature(facts, function.signature, inputs, kind).map(|_| ())
}
fn formal(
    facts: &Declarations,
    owner: DefId,
    index: usize,
    parameter: crate::sema::generics::Parameter,
) -> Result<TypeId, &'static str> {
    let Argument::Type(input) = parameter.argument else {
        return Err("source arithmetic requires a type formal");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(input) else {
        return Err("source arithmetic lost its actual formal");
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != owner.0
                || usize::try_from(p.index).ok() != Some(index)
                || p.kind != ParameterKind::Type { sized: true }
        })
    {
        return Err("source arithmetic changed its own sized type formal");
    }
    Ok(input)
}

fn signature(
    facts: &Declarations,
    signature: TypeId,
    inputs: [TypeId; 2],
    kind: SourceIntrinsic,
) -> Result<TypeId, &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("source subtraction requires its Rust signature without late regions");
    };
    if unsafe_ != (unchecked_operator(kind).is_some() || unchecked_shift(kind).is_some())
        || facts.types.list(parameters) != Some(&inputs)
    {
        return Err("source subtraction changed its safety or two same-type inputs");
    }
    let input = inputs[0];
    let valid = match kind {
        kind if overflowing_operator(kind).is_some() => {
            matches!(facts.types.kind(result), Some(Kind::Tuple(fields)) if facts.types.list(fields)==Some(&[input,TypeId::primitive(Primitive::Bool)]))
        }
        kind if unchecked_operator(kind).is_some()
            || wrapping_operator(kind).is_some()
            || unchecked_shift(kind).is_some() =>
        {
            result == input
        }
        _ => false,
    };
    if !valid {
        return Err("source subtraction changed its exact scalar or (scalar,bool) result");
    }
    Ok(result)
}
impl Catalog {
    pub(crate) fn subtraction_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<SubtractionSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature: selected_signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        if !scalar_arithmetic(kind) {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 32)
            .map_err(|_| "source subtraction selection work exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err(
                "source subtraction operation provenance requires its wider transport profile",
            );
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|i| facts.functions.get(*i))
            .ok_or("source subtraction declaration is missing")?;
        if function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
            || declaration.parent.is_some()
        {
            return Err("source subtraction changed its actual declaration");
        }
        validate(facts, *function, kind)?;
        self.validated_language(LangItem::Copy)?
            .ok_or("source subtraction requires its completed actual Copy header")?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("invalid source subtraction arguments")?;
        let Some(Argument::Type(input)) = args.next() else {
            return Err("source arithmetic requires its complete first type instance");
        };
        let rhs = if unchecked_shift(kind).is_some() {
            let Some(Argument::Type(rhs)) = args.next() else {
                return Err("source shift requires its complete second type instance");
            };
            if !matches!(facts.types.kind(rhs),Some(Kind::Primitive(p)) if p.integer().is_some()) {
                return Err("source shift amount requires a closed integer type");
            }
            rhs
        } else {
            input
        };
        if args.next().is_some() {
            return Err("source arithmetic has excess instance arguments");
        }
        if !matches!(facts.types.kind(input),Some(Kind::Primitive(p)) if p.integer().is_some()
            || wrapping_operator(kind).is_some() && matches!(p, Primitive::F32 | Primitive::F64))
        {
            return Err("source arithmetic execution requires its supported closed scalar type");
        }
        let output = signature(facts, selected_signature, [input, rhs], kind)?;
        let roots =
            super::intrinsic_contracts::item_roots(&facts.types, [input, rhs, output], |work| {
                crate::sema::header_attributes::charge_source_contract_query(facts, work)
                    .map_err(|_| "source subtraction opening work exhausted")
            })?
            .ok_or("source subtraction has no canonical caller roots")?;
        if roots != [input, rhs, output] {
            return Err("closed source subtraction roots changed through their callable binder");
        }
        Ok(Some(SubtractionSelection {
            definition,
            signature: selected_signature,
            input,
            rhs,
            output,
            kind,
        }))
    }
    pub(crate) fn require_subtraction_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &SubtractionSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, kind, _)) = self.intrinsic_record(selected.definition)? else {
            return Err("source subtraction lost its attributed declaration");
        };
        if kind != selected.kind
            || signature(
                facts,
                selected.signature,
                [selected.input, selected.rhs],
                kind,
            )? != selected.output
        {
            return Err("source subtraction changed its selected signature or intrinsic kind");
        }
        let header = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source subtraction has no current source header")?;
        if header
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(header.must_use, ContractKnowledge::Unknown)
            || matches!(header.inline, ContractKnowledge::Unknown)
        {
            return Err("source subtraction requires its positive actual source header");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/subtracting_source.rs"]
mod tests;
