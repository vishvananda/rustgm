//! Actual source/member seals reuse the existing configured-owner ledger.
use super::{Contracts, Declaration, Declarations, Error, LangItem, Report, header_attributes};
use crate::{
    sema::compiler_items::headers::{Flags, Trait},
    types::{ItemId, Kind, TypeId, projection},
};

pub(super) struct Plan {
    pub trait_: Declaration,
    pub member: Declaration,
    pub self_type: TypeId,
    pub projection: projection::Application,
}

pub(super) fn plan(report: &Report, facts: &Declarations) -> Result<Option<Plan>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let trait_ = catalog.language(LangItem::DiscriminantKind);
    let member = catalog.language(LangItem::DiscriminantType);
    if trait_.is_none() && member.is_none() {
        return Ok(None);
    }
    header_attributes::charge_source_contract_query(facts, 3)?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let trait_ =
        trait_.ok_or_else(|| Error::unlocated("source discriminant requires its actual trait"))?;
    let error = |message| crate::sema::model::Declarations::error(report, trait_.origin, message);
    let member =
        member.ok_or_else(|| error("source discriminant requires its owned result declaration"))?;
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
    if header.written.parameters != 0
        || header.written.supertraits != 0
        || header.members != [member.definition]
    {
        return Err(error(
            "source DiscriminantKind requires its actual single associated result",
        ));
    }
    required_member(report, member)?;
    let contract = facts
        .associated
        .require_projection_header(member.definition)
        .map_err(error)?;
    let Some(Kind::Projection(projection)) = facts.types.kind(contract.projection) else {
        return Err(error(
            "source discriminant lacks its canonical result projection",
        ));
    };
    let plan = Plan {
        trait_,
        member,
        self_type: header.self_type,
        projection,
    };
    validate_member(facts, &plan).map_err(error)?;
    Ok(Some(plan))
}

fn required_member(report: &Report, member: Declaration) -> Result<(), Error> {
    let error = |message| Declarations::error(report, member.origin, message);
    if report
        .resolution
        .definitions
        .get(member.definition.0 as usize)
        .is_none_or(|d| {
            d.kind != crate::resolve::model::DefKind::TypeAlias || d.node != Some(member.origin)
        })
    {
        return Err(error(
            "source discriminant requires its actual type declaration",
        ));
    }
    let ast = &report.package.unit(member.origin.file).configured.ast;
    let crate::syntax::ast::Kind::Item(index) = ast.node(member.origin.node).kind else {
        return Err(error("source discriminant lost its actual result item"));
    };
    let item = ast.items[index as usize];
    if !matches!(
        item.kind,
        crate::syntax::item::ItemKind::TypeAlias { ty: None, .. }
    ) || !ast.list(item.generics).is_empty()
    {
        return Err(error(
            "source discriminant requires its actual required non-GAT result",
        ));
    }
    Ok(())
}

fn validate_member(facts: &Declarations, plan: &Plan) -> Result<(), &'static str> {
    if plan.member.parent != Some(plan.trait_.definition)
        || facts.associated.trait_owner(plan.member.definition) != Some(plan.trait_.definition)
        || facts.generics.parents.get(&plan.member.definition) != Some(&plan.trait_.definition)
        || facts
            .generics
            .parameters
            .get(&plan.member.definition)
            .is_some_and(|p| !p.is_empty())
    {
        return Err("source discriminant changed its actual required member owner or parameters");
    }
    if let Some(family) = facts.associated.families.get(&plan.member.definition)
        && (family.captures != 0 || family.total != 0 || family.requirements.is_none())
    {
        return Err("source discriminant result inputs are pending or outside its profile");
    }
    let schema = facts
        .types
        .projection_schema(ItemId(plan.member.definition.0))
        .ok_or("source discriminant lacks its actual result schema")?;
    if schema.trait_item != ItemId(plan.trait_.definition.0)
        || plan.projection.item != ItemId(plan.member.definition.0)
        || plan.projection.subject != plan.self_type
        || facts
            .types
            .trait_application(plan.projection.trait_application)
            .is_none_or(|a| {
                a.item != schema.trait_item
                    || facts
                        .types
                        .arguments(a.arguments)
                        .is_none_or(|a| a.len() != 0)
            })
        || facts
            .types
            .arguments(plan.projection.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("source discriminant changed its complete projection identity");
    }
    Ok(())
}

impl Contracts {
    pub(in crate::sema::compiler_items) fn current_facts(
        &self,
        facts: &Declarations,
    ) -> Result<(), Error> {
        header_attributes::charge_source_contract_query(facts, 4)?;
        if !facts.types.has_identity(&self.arena) {
            return Err(Error::unlocated(
                "source discriminant belongs to another type arena",
            ));
        }
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| Error::unlocated("source discriminant lost its catalog"))?;
        if catalog.language(LangItem::DiscriminantKind) != Some(self.trait_)
            || catalog.language(LangItem::DiscriminantType) != Some(self.member)
        {
            return Err(Error::unlocated(
                "source discriminant changed its actual role identities",
            ));
        }
        for (declaration, image) in [self.trait_, self.member].into_iter().zip(&self.images) {
            header_attributes::require_recorded_pointee_image(facts, declaration, image)?;
        }
        let contract = facts
            .associated
            .require_projection_header(self.member.definition)
            .map_err(Error::unlocated)?;
        if facts.types.kind(contract.projection) != Some(Kind::Projection(self.projection)) {
            return Err(Error::unlocated(
                "source discriminant declaration projection changed",
            ));
        }
        validate_member(
            facts,
            &Plan {
                trait_: self.trait_,
                member: self.member,
                self_type: self.self_type,
                projection: self.projection,
            },
        )
        .map_err(Error::unlocated)
    }

    pub(super) fn current_report(
        &self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        self.current_facts(facts)?;
        let current = plan(report, facts)?
            .ok_or_else(|| Error::unlocated("source discriminant role disappeared"))?;
        if current.trait_ != self.trait_
            || current.member != self.member
            || current.self_type != self.self_type
            || current.projection != self.projection
        {
            return Err(Error::unlocated(
                "source discriminant current header changed",
            ));
        }
        for (declaration, image) in [self.trait_, self.member].into_iter().zip(&self.images) {
            let actual = header_attributes::pointee_source_image(facts, report, declaration)?;
            if !image.same(&actual) {
                return Err(Error::unlocated(
                    "source discriminant changed its source incarnation",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn application(
        &self,
        facts: &Declarations,
        actual: projection::Application,
    ) -> Result<(), Error> {
        let parent = facts
            .types
            .trait_application(actual.trait_application)
            .ok_or_else(|| Error::unlocated("source discriminant has invalid parent inputs"))?;
        if actual.item != ItemId(self.member.definition.0)
            || parent.item != ItemId(self.trait_.definition.0)
            || facts
                .types
                .arguments(parent.arguments)
                .is_none_or(|a| a.len() != 0)
            || facts
                .types
                .arguments(actual.arguments)
                .is_none_or(|a| a.len() != 0)
            || facts.types.kind(actual.subject).is_none()
        {
            return Err(Error::unlocated(
                "source discriminant request differs from its actual complete member",
            ));
        }
        Ok(())
    }
}
