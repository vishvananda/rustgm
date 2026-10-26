//! Borrow complete actual declaration contracts; no representation grants a role.
use super::{Catalog, Declaration, Declarations, Error, LangItem, Report};
use crate::{
    resolve::model::DefKind,
    sema::compiler_items::headers::{Flags, Trait},
    support::constness::Constness,
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{ItemId, Kind, TypeId, arguments::Argument, projection},
};

#[derive(Clone, Copy)]
pub(super) struct Plan {
    pub trait_: Declaration,
    pub member: Declaration,
    pub nominal: Option<Declaration>,
    pub self_type: TypeId,
    pub projection: projection::Application,
    pub schema: projection::Schema,
    pub attributes: crate::resolve::bindings::HeaderAttributes,
    pub denied_implementations: bool,
}
pub(super) fn plan(report: &Report, facts: &Declarations) -> Result<Option<Plan>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let trait_ = catalog.language(LangItem::PointeeTrait);
    let member = catalog.language(LangItem::MetadataType);
    let nominal = catalog.language(LangItem::DynMetadata);
    if trait_.is_none() && member.is_none() && nominal.is_none() {
        return Ok(None);
    }
    crate::sema::header_attributes::charge_source_contract_query(facts, 3)?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let trait_ = trait_
        .ok_or_else(|| Error::unlocated("source metadata requires its actual Pointee trait"))?;
    let error = |message| Declarations::error(report, trait_.origin, message);
    let member =
        member.ok_or_else(|| error("source Pointee requires its actual Metadata declaration"))?;
    let header = Trait::source(report, facts, trait_).map_err(error)?;
    header
        .profile(
            &facts.types,
            Flags {
                unsafe_: false,
                auto_: false,
                constant: false,
            },
            0,
        )
        .map_err(error)?;
    if header.written.parameters != 0 {
        return Err(error(
            "source Pointee parameters are outside the supported declaration profile",
        ));
    }
    if header.members != [member.definition] {
        return Err(Declarations::error(
            report,
            member.origin,
            "source Metadata role ownership is outside the supported Pointee declaration profile",
        ));
    }
    let pointee = catalog
        .header_role(LangItem::PointeeSized)
        .map_err(error)?
        .ok_or_else(|| error("source Pointee requires its actual PointeeSized role"))?;
    crate::sema::header_attributes::charge_source_contract_query(facts, header.inputs.len())?;
    if header.written.supertraits != 1
        || !header.requires(&facts.types, pointee.definition, None, Constness::Ordinary)
    {
        return Err(error(
            "source Pointee requires its actual direct PointeeSized bound",
        ));
    }
    let (projection, schema) = metadata(report, facts, trait_, member, header.self_type)?;
    if let Some(nominal) = nominal {
        dyn_metadata(report, facts, catalog, nominal)?;
    }
    Ok(Some(Plan {
        trait_,
        member,
        nominal,
        self_type: header.self_type,
        projection,
        schema,
        attributes: header.attributes,
        denied_implementations: report
            .resolution
            .source_bindings
            .as_deref()
            .is_some_and(|reserved| reserved.denies_implementation(trait_.definition)),
    }))
}
fn metadata(
    report: &Report,
    facts: &Declarations,
    trait_: Declaration,
    member: Declaration,
    self_type: TypeId,
) -> Result<(projection::Application, projection::Schema), Error> {
    let error = |message| Declarations::error(report, member.origin, message);
    if member.parent != Some(trait_.definition)
        || facts.associated.trait_owner(member.definition) != Some(trait_.definition)
        || facts.generics.parents.get(&member.definition) != Some(&trait_.definition)
        || report
            .resolution
            .definitions
            .get(member.definition.0 as usize)
            .is_none_or(|d| d.kind != DefKind::TypeAlias || d.node != Some(member.origin))
    {
        return Err(error(
            "source Metadata requires its actual Pointee associated owner",
        ));
    }
    let unit = report.package.unit(member.origin.file);
    let Syntax::Item(index) = unit.configured.ast.node(member.origin.node).kind else {
        return Err(error("source Metadata has no actual type item"));
    };
    let item = unit.configured.ast.items[index as usize];
    if !matches!(item.kind, ItemKind::TypeAlias { ty: None, .. })
        || !unit.configured.ast.list(item.generics).is_empty()
    {
        return Err(error(
            "source Metadata profile requires a required type without own parameters or default",
        ));
    }
    match facts.associated.families.get(&member.definition) {
        Some(family)
            if family.captures != 0
                || family.total != 0
                || family.requirements.is_none()
                || facts
                    .generics
                    .parameters
                    .get(&member.definition)
                    .is_none_or(|p| !p.is_empty()) =>
        {
            return Err(error(
                "source Metadata profile has incomplete or unsupported family inputs",
            ));
        }
        None if !unit.configured.ast.list(item.where_clause).is_empty() => {
            return Err(error(
                "source Metadata written family inputs have not completed",
            ));
        }
        _ => {}
    }
    let contract = facts
        .associated
        .require_projection_header(member.definition)
        .map_err(error)?;
    let Some(Kind::Projection(application)) = facts.types.kind(contract.projection) else {
        return Err(error("source Metadata has no actual canonical projection"));
    };
    let schema = facts
        .types
        .projection_schema(ItemId(member.definition.0))
        .ok_or_else(|| error("source Metadata has no canonical declaration schema"))?;
    if schema.trait_item != ItemId(trait_.definition.0)
        || application.item != ItemId(member.definition.0)
        || application.subject != self_type
        || facts
            .types
            .trait_application(application.trait_application)
            .is_none_or(|a| {
                a.item != schema.trait_item
                    || facts
                        .types
                        .arguments(a.arguments)
                        .is_none_or(|a| a.len() != 0)
            })
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err(error(
            "source Metadata projection changed its actual complete owner or arguments",
        ));
    }
    Ok((application, schema))
}
fn dyn_metadata(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let unit = report.package.unit(declaration.origin.file);
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| error("source DynMetadata lost its definition"))?;
    let Syntax::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err(error("source DynMetadata requires an actual structure"));
    };
    if actual.kind != DefKind::Struct
        || actual.node != Some(declaration.origin)
        || declaration.parent.is_some()
        || !matches!(
            unit.configured.ast.items[index as usize].kind,
            ItemKind::Struct { .. }
        )
    {
        return Err(error(
            "source DynMetadata requires an actual module structure",
        ));
    }
    let target = nominal_formal(report, facts, declaration)?;
    let pointee = catalog
        .header_role(LangItem::PointeeSized)
        .map_err(error)?
        .ok_or_else(|| error("source DynMetadata requires actual PointeeSized"))?;
    crate::sema::header_attributes::charge_source_contract_query(
        facts,
        facts
            .generics
            .predicates
            .get(&declaration.definition)
            .map_or(0, Vec::len),
    )?;
    let found = facts
        .generics
        .declared_predicates_in_scope(declaration.definition)
        .any(|clause| {
            let crate::types::parameters::clauses::Clause::Predicate(predicate) = clause.value
            else {
                return false;
            };
            predicate.ty == target
                && predicate.bound.trait_parts().is_some_and(|(id, c)| {
                    c == Constness::Ordinary
                        && facts.types.trait_application(id).is_some_and(|a| {
                            a.item.0 == pointee.definition.0
                                && facts
                                    .types
                                    .arguments(a.arguments)
                                    .is_some_and(|a| a.len() == 0)
                        })
                })
        });
    if !found {
        return Err(error(
            "source DynMetadata requires its real PointeeSized formal",
        ));
    }
    Ok(())
}

fn nominal_formal(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
) -> Result<TypeId, Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let parameters = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .ok_or_else(|| error("source DynMetadata parameters are not complete"))?;
    let [parameter] = parameters.as_slice() else {
        return Err(error(
            "source DynMetadata profile requires one actual type formal",
        ));
    };
    let Argument::Type(target) = parameter.argument else {
        return Err(error("source DynMetadata formal must be a type"));
    };
    if parameter.default.is_some()
        || parameter.anonymous
        || !matches!(facts.types.kind(target), Some(Kind::Parameter(id)) if facts.types.parameter(id).is_some_and(|p| p.owner == ItemId(declaration.definition.0) && p.index == 0))
    {
        return Err(error(
            "source DynMetadata formal changed its actual owner or default",
        ));
    }
    Ok(target)
}
