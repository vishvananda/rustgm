//! Composition authenticates a source call; metadata never grants pointee access.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{
        header_attributes::ContractKnowledge,
        model::{Declarations, Function},
    },
    support::bindings::{LangItem, SourceIntrinsic},
    types::{
        Kind, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::ParameterKind,
    },
};
/// A temporary direct-call selection, with no persistent caller permission.
pub(crate) struct RawPartsSelection {
    definition: DefId,
    signature: TypeId,
    output: TypeId,
    instance: Instance,
}
impl RawPartsSelection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn output(&self) -> TypeId {
        self.output
    }
    /// The data and metadata types as the caller writes them. The signature is a
    /// binder of its own and names a lifetime of the caller one level deeper.
    pub(crate) fn inputs(&self, facts: &Declarations) -> Result<[TypeId; 2], &'static str> {
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(self.signature) else {
            return Err("source aggregate_raw_ptr lost its callable signature");
        };
        let [data, meta] = facts
            .types
            .list(parameters)
            .ok_or("source aggregate_raw_ptr lost its inputs")?
        else {
            return Err("source aggregate_raw_ptr requires exactly two inputs");
        };
        super::intrinsic_contracts::item_roots(&facts.types, [*data, *meta], |steps| {
            crate::sema::header_attributes::charge_source_contract_query(facts, steps)
                .map_err(|_| "source aggregate_raw_ptr instance work exhausted")
        })?
        .ok_or("source aggregate_raw_ptr inputs lost their caller types")
    }
    /// A body Call's record holds its roots in the caller, unlike the item signature.
    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        self.inputs(facts).is_ok_and(|inputs| {
            super::intrinsic_contracts::opened_signature_matches(
                &facts.types,
                signature,
                &inputs,
                self.output,
                false,
                Convention::RUST,
            )
        })
    }
    pub(crate) fn current(&self, facts: &Declarations) -> Result<(), &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("source aggregate_raw_ptr lost its actual item");
        };
        let mut actual = facts
            .types
            .arguments(arguments)
            .ok_or("source aggregate_raw_ptr lost its complete arguments")?;
        let [data, meta] = self.inputs(facts)?;
        if item.0 != self.definition.0
            || signature != self.signature
            || actual.next() != Some(Argument::Type(self.output))
            || actual.next() != Some(Argument::Type(data))
            || actual.next() != Some(Argument::Type(meta))
            || actual.next().is_some()
        {
            return Err("source aggregate_raw_ptr changed its complete instance or signature");
        }
        Ok(())
    }
}
pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source aggregate_raw_ptr requires a safe Rust signature without late regions");
    };
    let formals = facts
        .generics
        .parameters
        .get(&function.definition)
        .ok_or("source aggregate_raw_ptr requires three owned type formals")?;
    if formals.len() != 3
        || !facts.is_const_function(function.definition)
        || function.body.is_some()
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_some_and(|r| r.total != 0)
    {
        return Err(
            "source aggregate_raw_ptr requires a const bodyless declaration with three owned types",
        );
    }
    let mut types = [TypeId::UNIT; 3];
    for (index, formal) in formals.iter().enumerate() {
        let Argument::Type(ty) = formal.argument else {
            return Err("source aggregate_raw_ptr requires only type formals");
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("source aggregate_raw_ptr lost a formal identity");
        };
        if facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != function.definition.0
                || p.index as usize != index
                || !matches!(p.kind, ParameterKind::Type { sized: true })
        }) {
            return Err("source aggregate_raw_ptr requires its ordered Sized P D M formals");
        }
        types[index] = ty;
    }
    if result != types[0] || facts.types.list(parameters) != Some(&types[1..]) {
        return Err("source aggregate_raw_ptr requires exactly (D,M) to its own P");
    }
    Ok(())
}
impl Catalog {
    fn raw_parts_header(
        &self,
        facts: &Declarations,
        definition: DefId,
        origin: BodyOrigin,
    ) -> Result<bool, &'static str> {
        if let Some((declaration, kind, original)) = self.intrinsic_record(definition)? {
            if kind != SourceIntrinsic::AggregateRawPtr {
                return Ok(false);
            }
            if origin.source().is_none()
                || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
            {
                return Err(
                    "imported aggregate_raw_ptr requires transported source operation provenance",
                );
            }
            let function = facts
                .function_indices
                .get(&definition)
                .and_then(|&i| facts.functions.get(i))
                .ok_or("source aggregate_raw_ptr has no completed declaration")?;
            if declaration.parent.is_some()
                || function.definition != definition
                || function.node != declaration.origin
                || function.signature != original
            {
                return Err("source aggregate_raw_ptr completed declaration changed");
            }
            crate::sema::header_attributes::charge_source_contract_query(facts, 16)
                .map_err(|_| "source aggregate_raw_ptr query budget exhausted")?;
            validate(facts, *function)?;
            return Ok(true);
        }
        let Some(imported) = self.imported_declared_intrinsic(definition) else {
            return Ok(false);
        };
        if imported.kind != SourceIntrinsic::AggregateRawPtr {
            return Ok(false);
        }
        let actual = facts
            .declaration_header(definition)?
            .ok_or("imported aggregate_raw_ptr lost its canonical declaration")?;
        if actual.declaration != imported.origin
            || actual.signature != imported.signature
            || actual.body.is_some()
            || !facts.is_const_function(definition)
        {
            return Err("imported aggregate_raw_ptr changed its canonical declaration");
        }
        Ok(true)
    }
    pub(crate) fn raw_parts_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<RawPartsSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        if !self.raw_parts_header(facts, definition, origin)? {
            return Ok(None);
        }
        let mut actual = facts
            .types
            .arguments(arguments)
            .ok_or("source aggregate_raw_ptr has invalid arguments")?;
        let (
            Some(Argument::Type(output)),
            Some(Argument::Type(data)),
            Some(Argument::Type(meta)),
            None,
        ) = (actual.next(), actual.next(), actual.next(), actual.next())
        else {
            return Err("source aggregate_raw_ptr requires its complete three-type instance");
        };
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("selected source aggregate_raw_ptr changed its safe Rust signature");
        };
        let Some(&[first, second]) = facts.types.list(parameters) else {
            return Err("selected source aggregate_raw_ptr input arity changed");
        };
        // Arguments inhabit the caller. The signature is a binder of its own, so a
        // lifetime of an enclosing binder appears in it one level deeper.
        let roots = super::intrinsic_contracts::item_roots(
            &facts.types,
            [first, second, result],
            |steps| {
                crate::sema::header_attributes::charge_source_contract_query(facts, steps)
                    .map_err(|_| "source aggregate_raw_ptr instance work exhausted")
            },
        )?;
        if roots != Some([data, meta, output]) {
            return Err("selected source aggregate_raw_ptr differs from its complete instance");
        }
        if !matches!(facts.types.kind(output), Some(Kind::Pointer { .. }))
            || !matches!(facts.types.kind(data), Some(Kind::Pointer { .. }))
        {
            return Err("source aggregate_raw_ptr requires actual raw result and data pointers");
        }
        for role in [LangItem::PointeeTrait, LangItem::MetadataType] {
            self.validated_role(role)?
                .ok_or("source aggregate_raw_ptr metadata role is incomplete")?;
        }
        Ok(Some(RawPartsSelection {
            definition,
            signature,
            output,
            instance,
        }))
    }
    pub(crate) fn require_raw_parts_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &RawPartsSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        selected.current(facts)?;
        super::pointee::authenticate(report, facts)
            .map_err(|_| "source aggregate_raw_ptr lost its current Pointee source image")?;
        let Some((declaration, SourceIntrinsic::AggregateRawPtr, _)) =
            self.intrinsic_record(selected.definition)?
        else {
            let Some(imported) = self.imported_declared_intrinsic(selected.definition) else {
                return Err("source aggregate_raw_ptr lost its actual intrinsic declaration");
            };
            if imported.kind != SourceIntrinsic::AggregateRawPtr
                || !facts.is_const_function(selected.definition)
                || facts
                    .declaration_header(selected.definition)?
                    .is_none_or(|actual| {
                        actual.declaration != imported.origin || actual.body.is_some()
                    })
            {
                return Err("imported aggregate_raw_ptr changed its canonical declaration");
            }
            return Ok(());
        };
        let header = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source aggregate_raw_ptr lost its current source header")?;
        if header
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(header.must_use, ContractKnowledge::Unknown)
            || matches!(header.inline, ContractKnowledge::Unknown)
        {
            return Err("source aggregate_raw_ptr requires its positive source header contract");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/compose_operands.rs"]
mod tests;
