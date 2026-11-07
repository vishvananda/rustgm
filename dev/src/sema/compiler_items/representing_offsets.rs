//! Field offset declaration data borrows actual Field identity and ordinary call proof.
use super::{Catalog, field_protocol};
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    support::{
        bindings::{LangItem, SourceIntrinsic},
        constness::Constness,
    },
    types::{
        Kind, Primitive, TypeId, arguments::Argument, instance::Instance,
        parameters::clauses::Clause,
    },
};

pub(crate) fn field_offset_intrinsic(kind: SourceIntrinsic) -> bool {
    matches!(kind, SourceIntrinsic::Declaration(identity) if identity.name() == "field_offset")
}

/// Physical coordinates only; every use must authenticate its checked caller again.
pub(crate) struct FieldOffsetSelection {
    pub definition: DefId,
    pub coordinates: field_protocol::Coordinates,
}

impl Catalog {
    pub(crate) fn field_offset_selection(
        &self,
        facts: &Declarations,
        target: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<FieldOffsetSelection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(target.ty())
        else {
            return Ok(None);
        };
        let Some((declaration, SourceIntrinsic::Declaration(identity), expected)) =
            self.intrinsic_record(DefId(item.0))?
        else {
            return Ok(None);
        };
        if identity.name() != "field_offset" {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source field_offset selection budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("imported field_offset operation contracts are not transported");
        }
        let function = facts
            .function_indices
            .get(&DefId(item.0))
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source field_offset lost its completed declaration")?;
        if declaration.parent.is_some()
            || function.node != declaration.origin
            || function.signature != expected
            || signature != expected
            || function.body.is_some()
            || !facts.is_const_function(function.definition)
            || !matches!(
                facts.types.kind(signature),
                Some(Kind::Function {
                    bound_regions: 0,
                    ..
                })
            )
            || facts
                .lifetimes
                .functions
                .get(&function.definition)
                .is_none_or(|scheme| scheme.total != 0)
        {
            return Err("source field_offset changed its actual const bodyless declaration");
        }
        super::prototype_validation::validate(facts, *function, identity)?;
        self.field_offset_constraints(facts, function.definition)?;
        let mut arguments = facts
            .types
            .arguments(arguments)
            .ok_or("source field_offset lost its full arguments")?;
        let (Some(Argument::Type(subject)), None) = (arguments.next(), arguments.next()) else {
            return Err("source field_offset requires exactly one actual type argument");
        };
        let coordinates = field_protocol::coordinates(facts, subject)
            .map_err(|_| "source field_offset lost its actual Field protocol or coordinates")?;
        Ok(Some(FieldOffsetSelection {
            definition: function.definition,
            coordinates,
        }))
    }

    fn field_offset_constraints(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let parameters = facts
            .generics
            .parameters
            .get(&definition)
            .ok_or("source field_offset lost its formal")?;
        let [parameter] = parameters.as_slice() else {
            return Err("source field_offset requires its one own formal");
        };
        let Argument::Type(formal) = parameter.argument else {
            return Err("source field_offset requires its type formal");
        };
        let field = self
            .language(LangItem::Field)
            .ok_or("source field_offset requires the actual Field trait")?;
        let mut found = false;
        for clause in facts.generics.declared_predicates(definition) {
            crate::sema::header_attributes::charge_source_contract_query(facts, 1)
                .map_err(|_| "source field_offset constraint budget exhausted")?;
            let Clause::Predicate(predicate) = clause.value else {
                continue;
            };
            let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
                continue;
            };
            let application = facts
                .types
                .trait_application(application)
                .ok_or("source field_offset lost its defining trait application")?;
            if predicate.ty == formal
                && application.item.0 == field.definition.0
                && facts
                    .types
                    .arguments(application.arguments)
                    .is_some_and(|args| args.len() == 0)
            {
                found = true;
            }
        }
        // The checked caller proves every actual defining clause, including
        // elaborated supertraits. This descriptor only requires the real F:Field
        // relationship; it cannot replace those other ordinary obligations.
        if !found {
            return Err("source field_offset requires its actual F:Field bound");
        }
        Ok(())
    }

    pub(crate) fn require_field_offset_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &FieldOffsetSelection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        super::authenticate_field(report, facts)
            .map_err(|_| "source field_offset Field source or arena changed")?;
        let Some((declaration, kind, signature)) = self.intrinsic_record(selected.definition)?
        else {
            return Err("source field_offset lost its attributed declaration");
        };
        if !field_offset_intrinsic(kind)
            || !matches!(facts.types.kind(signature), Some(Kind::Function { result, .. })
                if result == TypeId::primitive(Primitive::Usize))
        {
            return Err("source field_offset changed its registered operation");
        }
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source field_offset current source header changed")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
        {
            return Err("source field_offset requires its positive completed source header");
        }
        Ok(())
    }
}
