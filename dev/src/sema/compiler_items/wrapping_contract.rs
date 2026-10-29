//! Signed wrapping offset retains its actual source declaration and closed pointer type.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::SourceIntrinsic,
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};

/// Substituted declaration data; no unsafe, const, memory or caller permission.
pub(crate) struct WrappingOffsetSelection {
    definition: DefId,
    signature: TypeId,
    pointer: TypeId,
    element: TypeId,
}
impl WrappingOffsetSelection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn pointer(&self) -> TypeId {
        self.pointer
    }
    pub(crate) const fn element(&self) -> TypeId {
        self.element
    }
    pub(crate) fn inputs(&self) -> [TypeId; 2] {
        [self.pointer, TypeId::primitive(Primitive::Isize)]
    }
    pub(crate) fn opened_signature_matches(
        &self,
        store: &crate::types::Store,
        signature: TypeId,
    ) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            store,
            signature,
            &self.inputs(),
            self.pointer,
            true,
            Convention::RUST,
        )
    }
}

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let generic = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or("source arith_offset requires its own Sized type parameter")?;
    if generic.len() != 1
        || function.body.is_some()
        || !facts.is_const_function(function.definition)
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
    {
        return Err(
            "source arith_offset requires one own type and a const bodyless declaration without regions",
        );
    }
    let Argument::Type(element) = generic[0].argument else {
        return Err("source arith_offset requires a type parameter");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(element) else {
        return Err("source arith_offset lost its actual type parameter");
    };
    if facts.types.parameter(id).is_none_or(|p| {
        p.owner.0 != function.definition.0
            || p.index != 0
            || !matches!(p.kind, ParameterKind::Type { sized: true })
    }) {
        return Err("source arith_offset requires its own ordered Sized type parameter");
    }
    pointer_signature(facts, function.signature, element).map(|_| ())
}
fn pointer_signature(
    facts: &Declarations,
    signature: TypeId,
    element: TypeId,
) -> Result<TypeId, &'static str> {
    pointer_types(facts, signature_parts(facts, signature)?, element)
}
fn signature_parts(facts: &Declarations, signature: TypeId) -> Result<[TypeId; 3], &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: true,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("source arith_offset requires an unsafe Rust signature without late regions");
    };
    let Some(&[pointer, count]) = facts.types.list(parameters) else {
        return Err("source arith_offset requires exactly two value parameters");
    };
    Ok([pointer, count, result])
}
fn pointer_types(
    facts: &Declarations,
    [pointer, count, result]: [TypeId; 3],
    element: TypeId,
) -> Result<TypeId, &'static str> {
    if facts.types.kind(pointer)
        != Some(Kind::Pointer {
            mutable: false,
            pointee: element,
        })
        || count != TypeId::primitive(Primitive::Isize)
        || result != pointer
    {
        return Err("source arith_offset requires (*const T,isize)->*const T");
    }
    Ok(pointer)
}
impl Catalog {
    pub(crate) fn wrapping_offset_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<WrappingOffsetSelection>, &'static str> {
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
        if kind != SourceIntrinsic::ArithOffset {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source arith_offset selection work budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("source arith_offset operation provenance is not transported");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("source arith_offset has no completed declaration")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source arith_offset completed declaration changed");
        }
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("invalid arith_offset instance arguments")?;
        let (Some(Argument::Type(element)), None) = (args.next(), args.next()) else {
            return Err("source arith_offset requires its complete one-type instance");
        };
        if facts.types.contains_parameters(element) != Some(false)
            || facts.types.contains_projections(element) != Some(false)
            || facts.types.is_sized(element) != Some(true)
        {
            return Err("source arith_offset execution requires its closed Sized element");
        }
        let roots = super::intrinsic_contracts::item_roots(
            &facts.types,
            signature_parts(facts, signature)?,
            |steps| {
                crate::sema::header_attributes::charge_source_contract_query(facts, steps)
                    .map_err(|_| "source arith_offset instance query work exhausted")
            },
        )?
        .ok_or("source arith_offset has no canonical caller input/result types")?;
        let pointer = pointer_types(facts, roots, element)?;
        Ok(Some(WrappingOffsetSelection {
            definition,
            signature,
            pointer,
            element,
        }))
    }
    pub(crate) fn require_wrapping_offset_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &WrappingOffsetSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, SourceIntrinsic::ArithOffset, _)) =
            self.intrinsic_record(selected.definition)?
        else {
            return Err("source arith_offset lost its attributed declaration");
        };
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source arith_offset lost current source-header authentication")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("source arith_offset requires its positive current source-header site");
        }
        let unit = report.package.unit(declaration.origin.file);
        let crate::syntax::ast::Kind::Item(index) =
            unit.configured.ast.node(declaration.origin.node).kind
        else {
            return Err("source arith_offset lost its current function header");
        };
        let item = unit
            .configured
            .ast
            .items
            .get(index as usize)
            .ok_or("source arith_offset current header is stale")?;
        if !matches!(
            item.kind,
            crate::syntax::item::ItemKind::Function {
                flags: crate::syntax::item::FunctionFlags {
                    constant: true,
                    asynchronous: false,
                    safety: crate::syntax::item::Safety::Unsafe
                },
                abi: crate::syntax::item::Abi::Rust,
                body: None,
                ..
            }
        ) {
            return Err("source arith_offset changed its actual unsafe const declaration");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/signed_source.rs"]
mod tests;
