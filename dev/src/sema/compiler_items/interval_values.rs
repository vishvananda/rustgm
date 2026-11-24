//! Range tags authenticate ordinary source structs and a real inherent constructor.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(in crate::sema) fn complete_constructor(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let role = LangItem::RangeInclusiveNew;
    let Some(declaration) = super::headers::functions::inherent(report, facts, role)? else {
        return Ok(());
    };
    super::roles::complete_constructor(facts, role, declaration)
}

/// Authenticate the implicit legacy call; its actual body owns all storage.
pub(in crate::sema) fn source_constructor(
    report: &Report,
    facts: &Declarations,
    site: crate::resolve::unit::NodeKey,
) -> Result<crate::resolve::model::DefId, Error> {
    let error = |message| Declarations::error(report, site, message);
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("inclusive constructor requires its actual source catalog"))?;
    catalog.require_provider_report(report).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, 5)?;
    if catalog.uses_imported_roles() {
        return catalog
            .validated_role(LangItem::RangeInclusiveNew)
            .map_err(error)?
            .map(|role| role.definition)
            .ok_or_else(|| {
                error("inclusive syntax requires its actual imported source constructor")
            });
    }
    let nominal = catalog
        .validated_language(LangItem::RangeInclusiveStruct)
        .map_err(error)?
        .ok_or_else(|| error("inclusive syntax requires its actual source range declaration"))?;
    crate::sema::header_attributes::completed_source_image(facts, report, nominal.definition)?;
    // Recheck the current nominal producer facts as well as its completed role.
    if report
        .resolution
        .definitions
        .get(nominal.definition.0 as usize)
        .and_then(|definition| definition.node)
        != Some(nominal.origin)
    {
        return Err(error("inclusive range changed its actual nominal source"));
    }
    super::role_values::nominal(facts, nominal.definition, LangItem::RangeInclusiveStruct)
        .map_err(error)?;
    let declaration =
        super::headers::functions::inherent(report, facts, LangItem::RangeInclusiveNew)?
            .ok_or_else(|| error("inclusive syntax requires its actual source constructor"))?;
    crate::sema::header_attributes::completed_source_image(facts, report, declaration.definition)?;
    let header = facts
        .function_header(declaration.definition)
        .map_err(error)?
        .ok_or_else(|| error("inclusive constructor requires its completed header"))?;
    constructor_signature(facts, nominal.definition, header.signature).map_err(error)?;
    Ok(declaration.definition)
}

pub(super) fn imported(
    facts: &Declarations,
    catalog: &super::Catalog,
    steps: &mut usize,
    complete: &mut super::roles::RoleSet,
) -> Result<(), &'static str> {
    for role in LangItem::RANGE_NOMINALS {
        *steps = steps.saturating_add(4);
        let Some(declaration) = catalog.role(role)? else {
            continue;
        };
        super::role_values::nominal(facts, declaration.definition, role)?;
        complete.insert(role);
    }
    let Some(constructor) = catalog.role(LangItem::RangeInclusiveNew)? else {
        return Ok(());
    };
    if !complete.contains(LangItem::RangeInclusiveStruct) {
        return Err("imported range constructor lost its actual inclusive range role");
    }
    let nominal = catalog
        .role(LangItem::RangeInclusiveStruct)?
        .ok_or("imported inclusive range lost its bound declaration")?;
    *steps = steps.saturating_add(8);
    imported_constructor(facts, nominal, constructor)?;
    complete.insert(LangItem::RangeInclusiveNew);
    Ok(())
}

fn imported_constructor(
    facts: &Declarations,
    nominal: super::Role,
    constructor: super::Role,
) -> Result<(), &'static str> {
    use crate::sema::associated::OwnerKind;
    let owner = facts
        .associated
        .owner(constructor.definition)
        .and_then(|owner| facts.associated.get(owner))
        .ok_or("imported inclusive constructor lost its actual implementation")?;
    let header = facts
        .declaration_header(constructor.definition)?
        .ok_or("imported inclusive constructor lost its checked function header")?;
    let schema = facts
        .associated
        .schema(owner.self_definition)
        .ok_or("imported inclusive constructor lost its actual Self schema")?;
    let actual_self = matches!(
        facts.types.kind(schema.self_type),
        Some(crate::types::Kind::Adt(item, _)) if item.0 == nominal.definition.0
    );
    if !actual_self
        || !matches!(owner.kind, OwnerKind::ImportedInherent(_))
        || constructor.parent != Some(owner.self_definition)
        || header.declaration != constructor.origin
        || header.body.is_none()
    {
        return Err("imported inclusive constructor changed its actual owner or body");
    }
    constructor_signature(facts, nominal.definition, header.signature)
}

fn constructor_signature(
    facts: &Declarations,
    nominal: crate::resolve::model::DefId,
    signature: crate::types::TypeId,
) -> Result<(), &'static str> {
    use crate::types::Kind;
    if !matches!(facts.types.kind(signature), Some(Kind::Function { parameters, result, .. })
        if facts.types.list(parameters).map(<[_]>::len) == Some(2)
            && matches!(facts.types.kind(result), Some(Kind::Adt(owner, _))
                if owner.0 == nominal.0))
    {
        return Err("inclusive constructor requires two inputs and its actual range result");
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/inclusive_calls.rs"]
mod tests;
