//! Conservative value knowledge authenticates the source declaration and its Copy premise.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    support::bindings::{LangItem, SourceIntrinsic},
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
    },
};

pub(crate) fn is_operation(kind: SourceIntrinsic) -> bool {
    matches!(kind, SourceIntrinsic::Declaration(identity) if identity.name() == "is_val_statically_known")
}

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
    input: TypeId,
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
            TypeId::primitive(Primitive::Bool),
            false,
            Convention::RUST,
        )
    }
}
impl Catalog {
    pub(crate) fn static_knowledge(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, &'static str> {
        let selected = self.knowledge_instance(facts, instance)?;
        if selected.is_some() && origin.source().is_none() {
            return Err("source value knowledge caller provenance is not transported");
        }
        Ok(selected)
    }
    pub(crate) fn knowledge_fallback(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<bool, &'static str> {
        let Some(selected) = self.knowledge_instance(facts, instance)? else {
            return Ok(false);
        };
        let function = facts
            .function_indices
            .get(&selected.definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source value knowledge fallback lost its actual function")?;
        if function.body.is_none() {
            return Err("source value knowledge requires its actual native fallback body");
        }
        Ok(true)
    }
    fn knowledge_instance(
        &self,
        facts: &Declarations,
        instance: Instance,
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
        if !is_operation(kind) {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 32)
            .map_err(|_| "source value knowledge selection work exhausted")?;
        if matches!(self.bindings, super::binding_views::Bindings::Imported(_)) {
            return Err("source value knowledge declaration provenance is not transported");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source value knowledge requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source value knowledge completed declaration changed");
        }
        let parameters = facts
            .generics
            .parameters
            .get(&definition)
            .ok_or("source value knowledge lost its own type formal")?;
        let [parameter] = parameters.as_slice() else {
            return Err("source value knowledge requires its single type formal");
        };
        let Argument::Type(formal) = parameter.argument else {
            return Err("source value knowledge changed its actual type formal");
        };
        super::compare_intrinsic::copy_premise(facts, definition, formal, self)
            .map_err(|_| "source value knowledge requires its actual ordinary Copy premise")?;
        self.validated_language(LangItem::Copy)?
            .ok_or("source value knowledge requires its completed actual Copy header")?;
        let input = knowledge_input(facts, arguments, signature)?;
        Ok(Some(Selection {
            definition,
            signature,
            input,
        }))
    }
    pub(crate) fn require_knowledge_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &Selection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, kind, _)) = self.intrinsic_record(selected.definition)? else {
            return Err("source value knowledge lost its attributed declaration");
        };
        if !is_operation(kind) {
            return Err("source value knowledge changed its intrinsic identity");
        }
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source value knowledge lost its current source header")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
            || !facts.is_const_function(selected.definition)
        {
            return Err("source value knowledge requires its actual completed const declaration");
        }
        Ok(())
    }
}
fn knowledge_input(
    facts: &Declarations,
    arguments: crate::types::GenericArgs,
    signature: TypeId,
) -> Result<TypeId, &'static str> {
    let mut args = facts
        .types
        .arguments(arguments)
        .ok_or("invalid source value knowledge arguments")?;
    let (Some(Argument::Type(input)), None) = (args.next(), args.next()) else {
        return Err("source value knowledge requires its complete one-type instance");
    };
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("source value knowledge changed its safe Rust signature");
    };
    let Some(&[parameter]) = facts.types.list(parameters) else {
        return Err("source value knowledge requires one actual value input");
    };
    let roots =
        super::intrinsic_contracts::item_roots(&facts.types, [parameter, result], |work| {
            crate::sema::header_attributes::charge_source_contract_query(facts, work)
                .map_err(|_| "source value knowledge opening work exhausted")
        })?
        .ok_or("source value knowledge has no canonical caller roots")?;
    if roots != [input, TypeId::primitive(Primitive::Bool)] {
        return Err("source value knowledge changed its exact (T) to bool signature");
    }
    Ok(input)
}
