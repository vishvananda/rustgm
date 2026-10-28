//! Full symbolic ownership, region and cleanup checking for source regressions.
use super::semantic;
use rustgm::{
    body, borrow,
    types::{Kind, copy::Copies},
};

pub fn check(source: &str) -> Result<Vec<borrow::loans::Analysis>, Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source).map_err(|e| format!("semantics: {e}"))?;
    let mut templates =
        body::templates::build(&report, &facts).map_err(|e| format!("template build: {e}"))?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)
        .map_err(|e| format!("promotion: {e}"))?;
    let contracts = body::regions::collect(&facts).map_err(|e| format!("contracts: {e}"))?;
    let contracts = borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
    )
    .map_err(|e| format!("regions: {e}"))?;
    verify(&facts, &templates, &contracts).map_err(|e| format!("cleanup: {e}").into())
}
pub fn verify(
    facts: &rustgm::sema::model::Declarations,
    templates: &body::templates::Templates,
    contracts: &borrow::regions::Analysis,
) -> Result<Vec<borrow::loans::Analysis>, Box<dyn std::error::Error>> {
    let mut copies = Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    let custom: Vec<_> = facts
        .destructors
        .implementations()
        .iter()
        .map(|d| d.item)
        .collect();
    Ok(borrow::loans::templates::verify(
        &mut environment,
        templates,
        contracts,
        &custom,
        |ty, selected| {
            use body::drops::{MemberPosition, MemberQuery};
            let Some(Kind::Adt(owner, _)) = facts.types.kind(ty) else {
                return Err("invalid selected nominal owner");
            };
            let id = match selected {
                MemberQuery::Field(body::ir::Selection::Nominal(field)) => facts
                    .members
                    .variant(field)
                    .ok_or("missing field variant")?,
                MemberQuery::Variant(variant) => {
                    if facts
                        .aggregates
                        .get(owner)
                        .is_none_or(|d| d.kind != rustgm::types::aggregate::Kind::Enum)
                    {
                        return Err("selected variant has no enum owner");
                    }
                    variant
                }
                MemberQuery::Field(_) => return Err("invalid selected nominal field"),
            };
            let variant = facts
                .members
                .variants
                .get(id.0 as usize)
                .ok_or("missing variant")?;
            if variant.owner != owner {
                return Err("foreign selected member");
            }
            match selected {
                MemberQuery::Field(body::ir::Selection::Nominal(field)) => {
                    Ok(MemberPosition::Field {
                        variant: variant.ordinal,
                        field: facts
                            .members
                            .ordinal(field)
                            .ok_or("missing field ordinal")?,
                    })
                }
                MemberQuery::Variant(_) => Ok(MemberPosition::Variant(variant.ordinal)),
                MemberQuery::Field(_) => Err("invalid selected nominal field"),
            }
        },
    )?)
}
