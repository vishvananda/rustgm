//! Actual source Src->Dst declarations retain their full signature and environment.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::SourceIntrinsic,
    syntax::{
        ast::Kind as Syntax,
        item::{Abi, FunctionFlags, GenericFlavor, ItemKind, Safety},
    },
    types::{
        Kind, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

/// Declaration and substituted type data, never unsafe/caller/execution permission.
pub(crate) struct TransmuteSelection {
    pub definition: DefId,
    pub signature: TypeId,
    pub source: TypeId,
    pub destination: TypeId,
}
impl TransmuteSelection {
    pub(crate) fn opened_signature_matches(
        &self,
        store: &crate::types::Store,
        signature: TypeId,
    ) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            store,
            signature,
            &[self.source],
            self.destination,
            true,
            Convention::RUST,
        )
    }
}

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: true,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source transmute requires an unsafe Rust signature without late regions");
    };
    let generic = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or("source transmute requires its two own Sized type parameters")?;
    if generic.len() != 2
        || !facts.is_const_function(function.definition)
        || function.body.is_some()
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
    {
        return Err(
            "source transmute requires two types and a const bodyless declaration without captured regions",
        );
    }
    let mut types = [TypeId::UNIT; 2];
    for (index, parameter) in generic.iter().enumerate() {
        let Argument::Type(ty) = parameter.argument else {
            return Err("source transmute requires only type parameters");
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("source transmute lost its actual type parameter");
        };
        if facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != function.definition.0
                || p.index as usize != index
                || !matches!(p.kind, ParameterKind::Type { sized: true })
        }) {
            return Err("source transmute requires its own ordered Sized parameters");
        }
        types[index] = ty;
    }
    if facts.types.list(parameters) != Some(types[..1].as_ref()) || result != types[1] {
        return Err("source transmute requires exactly Src by value and Dst result");
    }
    Ok(())
}

/// Arguments inhabit the caller; signature roots inhabit one further function
/// binder, including signatures with no own late regions. Nested locals remain
/// local. Canonical lookup cannot manufacture a type or a caller permission.
fn instance_types_match(
    store: &crate::types::Store,
    parameters: crate::types::ListId,
    result: TypeId,
    expected: [TypeId; 2],
    mut charge: impl FnMut(usize) -> Result<(), &'static str>,
) -> Result<bool, &'static str> {
    let Some(&[input]) = store.list(parameters) else {
        charge(1)?;
        return Err("source transmute requires its complete one-input signature");
    };
    Ok(super::intrinsic_contracts::item_roots(store, [input, result], charge)? == Some(expected))
}

impl Catalog {
    pub(crate) fn transmute_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<TransmuteSelection>, &'static str> {
        if let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty())
            && self.intrinsic_kind(DefId(item.0))? == Some(SourceIntrinsic::TransmuteUnchecked)
        {
            // Unchecked reinterpretation has no declaration-time size premise.
            return Ok(None);
        }
        self.reinterpretation_selection(facts, instance, origin)
    }

    pub(crate) fn reinterpretation_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<TransmuteSelection>, &'static str> {
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
        if !matches!(
            kind,
            SourceIntrinsic::Transmute | SourceIntrinsic::TransmuteUnchecked
        ) {
            return Ok(None);
        }
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("imported transmute requires transported source operation provenance");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("source transmute has no completed declaration")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source transmute completed declaration changed");
        }
        validate(facts, *function)?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("invalid transmute instance arguments")?;
        let (Some(Argument::Type(source)), Some(Argument::Type(destination)), None) =
            (args.next(), args.next(), args.next())
        else {
            return Err("source transmute requires its exact complete two-type instance");
        };
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("selected transmute changed its unsafe Rust signature");
        };
        if !instance_types_match(
            &facts.types,
            parameters,
            result,
            [source, destination],
            |steps| {
                crate::sema::header_attributes::charge_source_contract_query(facts, steps)
                    .map_err(|_| "source transmute instance query work exhausted")
            },
        )? {
            return Err("selected transmute signature differs from its complete instance");
        }
        Ok(Some(TransmuteSelection {
            definition,
            signature,
            source,
            destination,
        }))
    }

    pub(crate) fn require_transmute_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &TransmuteSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((
            declaration,
            SourceIntrinsic::Transmute | SourceIntrinsic::TransmuteUnchecked,
            _,
        )) = self.intrinsic_record(selected.definition)?
        else {
            return Err("source transmute lost its attributed declaration");
        };
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source transmute lost current source-header authentication")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("source transmute requires its positive current source-header site");
        }
        let unit = report.package.unit(declaration.origin.file);
        let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
            return Err("source transmute lost its current function item");
        };
        let item = unit
            .configured
            .ast
            .items
            .get(index as usize)
            .ok_or("stale source transmute item")?;
        let ItemKind::Function {
            flags,
            abi: Abi::Rust,
            parameters,
            body: None,
            ..
        } = item.kind
        else {
            return Err("source transmute requires its current bodyless Rust declaration");
        };
        if flags
            != (FunctionFlags {
                constant: true,
                asynchronous: false,
                safety: Safety::Unsafe,
            })
            || unit.configured.ast.list(parameters).len() != 1
            || unit.configured.ast.list(item.generics).len() != 2
            || !unit.configured.ast.list(item.where_clause).is_empty()
        {
            return Err("source transmute changed its current unsafe const header");
        }
        for &node in unit.configured.ast.list(item.generics) {
            let Syntax::Generic(index) = unit.configured.ast.node(node).kind else {
                return Err("source transmute lost its actual generic syntax");
            };
            let generic = unit
                .configured
                .ast
                .generics
                .get(index as usize)
                .ok_or("stale source transmute generic parameter")?;
            if generic.flavor != GenericFlavor::Type
                || generic.ty.is_some()
                || generic.default.is_some()
                || !unit.configured.ast.list(generic.bounds).is_empty()
            {
                return Err("source transmute requires its current implicit Sized type parameters");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/callee_regions.rs"]
mod borrowed_tests;

#[cfg(test)]
#[path = "../../../tests/invariants/unchecked_transmute.rs"]
mod unchecked_tests;
