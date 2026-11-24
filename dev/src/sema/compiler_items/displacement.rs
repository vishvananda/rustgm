//! Exact source intrinsic data never retains a caller or a field-result cache.
use super::Catalog;
use crate::{
    resolve::model::DefId,
    sema::model::{Declarations, Function},
    support::{
        bindings::{LangItem, SourceIntrinsic},
        constness::Constness,
    },
    types::{
        Kind, Primitive, TypeId,
        arguments::Argument,
        callable::Convention,
        instance::Instance,
        parameters::{ParameterKind, clauses::Clause},
    },
};

#[derive(Clone, Copy)]
pub(crate) struct OffsetSelection {
    pub definition: DefId,
    pub element: TypeId,
}

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    if function.body.is_some()
        || facts
            .lifetimes
            .functions
            .get(&function.definition)
            .is_none_or(|r| r.total != 0)
    {
        return Err("source offset_of requires its const bodyless (u32,u32)->usize declaration");
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source offset_of has no catalog")?;
    let language = catalog
        .language(LangItem::OffsetOf)
        .ok_or("source offset_of requires its actual language declaration")?;
    if language.definition != function.definition
        || language.origin != function.node
        || language.parent.is_some()
    {
        return Err("source offset_of intrinsic differs from its actual language declaration");
    }
    validate_header(facts, catalog, function.definition, function.signature)
}

pub(super) fn complete_provider(
    facts: &mut Declarations,
) -> Result<(), crate::resolve::package::Error> {
    let language = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::OffsetOf))
        .ok_or_else(|| {
            crate::resolve::package::Error::unlocated(
                "source offset_of lost its validated language declaration",
            )
        })?;
    super::roles::record_completion(facts, LangItem::OffsetOf, language)
}

pub(super) fn validate_imported(
    facts: &Declarations,
    catalog: &Catalog,
    role: super::Role,
    signature: TypeId,
    steps: &mut usize,
) -> Result<TypeId, &'static str> {
    if catalog.role(LangItem::OffsetOf)? != Some(role) || role.parent.is_some() {
        return Err("imported offset_of changed its actual language declaration");
    }
    *steps = steps.saturating_add(16);
    validate_header(facts, catalog, role.definition, signature)?;
    Ok(TypeId::primitive(Primitive::Usize))
}

fn validate_header(
    facts: &Declarations,
    catalog: &Catalog,
    definition: DefId,
    signature: TypeId,
) -> Result<(), &'static str> {
    let u32_ = TypeId::primitive(Primitive::U32);
    if !facts.is_const_function(definition)
        || !super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            &[u32_, u32_],
            TypeId::primitive(Primitive::Usize),
            false,
            Convention::RUST,
        )
    {
        return Err("source offset_of requires its const safe Rust (u32,u32)->usize signature");
    }
    let ty = formal(facts, definition)?;
    constraint(facts, catalog, definition, ty)
}

fn formal(facts: &Declarations, definition: DefId) -> Result<TypeId, &'static str> {
    let generic = facts
        .generics
        .parameters
        .get(&definition)
        .ok_or("source offset_of requires one own PointeeSized type formal")?;
    let [parameter] = generic.as_slice() else {
        return Err("source offset_of requires exactly one own type formal");
    };
    let Argument::Type(ty) = parameter.argument else {
        return Err("source offset_of requires a type formal");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("source offset_of lost its actual formal identity");
    };
    if facts.types.parameter(id).is_none_or(|p| {
        p.owner.0 != definition.0
            || p.index != 0
            || !matches!(p.kind, ParameterKind::Type { sized: false })
    }) {
        return Err("source offset_of requires its own PointeeSized formal");
    }
    Ok(ty)
}

fn constraint(
    facts: &Declarations,
    catalog: &Catalog,
    definition: DefId,
    ty: TypeId,
) -> Result<(), &'static str> {
    let sizing = catalog
        .sizing
        .ok_or("source offset_of requires completed sizing headers")?;
    let inputs = crate::sema::declaration_inputs::declared_own(
        &facts.generics,
        facts.crates.as_deref(),
        definition,
    )?;
    let mut inputs = inputs.iter();
    let (Some(input), None) = (inputs.next(), inputs.next()) else {
        return Err("source offset_of requires only its actual PointeeSized bound");
    };
    let Clause::Predicate(predicate) = input.value else {
        return Err("source offset_of does not declare quantified constraints");
    };
    let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Err("source offset_of requires its ordinary PointeeSized constraint");
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("source offset_of lost its formal trait constraint")?;
    if predicate.ty != ty
        || application.item.0 != sizing[2].0
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("source offset_of changed its actual PointeeSized constraint");
    }
    Ok(())
}

impl Catalog {
    pub(crate) fn require_offset_callable(
        &self,
        report: &crate::resolve::expansion::Report,
        facts: &Declarations,
        callable: TypeId,
        container: TypeId,
    ) -> Result<(), &'static str> {
        crate::sema::header_attributes::charge_source_contract_query(facts, 8)
            .map_err(|_| "offset callable contract budget exhausted")?;
        let Some(Kind::FunctionItem {
            item,
            signature,
            arguments,
        }) = facts.types.kind(callable)
        else {
            return Err("offset plan requires its actual source function item");
        };
        let definition = DefId(item.0);
        let checked_signature = self.offset_header(facts, definition)?;
        self.require_offset_report(report, facts, definition)?;
        let mut actual = facts
            .types
            .arguments(arguments)
            .ok_or("offset callable arguments disappeared")?;
        if signature != checked_signature
            || actual.len() != 1
            || actual.next() != Some(Argument::Type(container))
        {
            return Err("offset callable changed its actual container or signature");
        }
        Ok(())
    }
    pub(crate) fn require_offset_report(
        &self,
        report: &crate::resolve::expansion::Report,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        if self.uses_imported_roles() {
            self.offset_header(facts, definition)?;
            return Ok(());
        }
        let Some((declaration, SourceIntrinsic::OffsetOf, _)) =
            self.intrinsic_record(definition)?
        else {
            return Err("source offset_of lost its actual intrinsic declaration");
        };
        let current = facts
            .header_attribute_contracts(report, definition)
            .map_err(|_| "source offset_of source header changed")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(
                current.must_use,
                crate::sema::header_attributes::ContractKnowledge::Unknown
            )
            || matches!(
                current.inline,
                crate::sema::header_attributes::ContractKnowledge::Unknown
            )
        {
            return Err("source offset_of requires positive actual source-header completion");
        }
        Ok(())
    }
    pub(crate) fn offset_selection(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<Option<OffsetSelection>, &'static str> {
        let Some(selected) = self.selected_intrinsic(facts, instance)? else {
            return Ok(None);
        };
        if selected.kind != SourceIntrinsic::OffsetOf {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source offset_of selection budget exhausted")?;
        self.offset_header(facts, selected.definition)?;
        Ok(Some(OffsetSelection {
            definition: selected.definition,
            element: selected.element,
        }))
    }
    fn offset_header(
        &self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<TypeId, &'static str> {
        if self.uses_imported_roles() {
            let Some((
                crate::support::crates::generics::source::BodylessIntrinsic::OffsetOf,
                header,
            )) = self.imported_intrinsic(definition)
            else {
                return Err("imported offset_of requires its exact checked intrinsic header");
            };
            return Ok(header.signature);
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&i| facts.functions.get(i))
            .ok_or("offset callable has no completed header")?;
        validate(facts, *function)?;
        Ok(function.signature)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/displacement_calls.rs"]
mod tests;
