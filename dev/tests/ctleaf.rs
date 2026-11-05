//! Constructor coordinates authenticate actual member inventory without permission.
use crate::resolve as source;
use rustgm::{
    body::drops::{self, MemberPosition, MemberQuery},
    sema,
    types::{ItemId, Kind, TypeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn facts() -> Result<(
    rustgm::resolve::expansion::Report,
    sema::model::Declarations,
)> {
    let text = source::fixture("pa31/tests/spec/source-const-cleanup/qctor.rs")?;
    let report = source::source_provider(&text, "cleanup_constructor_queries")?;
    let facts = sema::declarations::analyze(&report)?;
    Ok((report, facts))
}
fn nominal(
    report: &rustgm::resolve::expansion::Report,
    facts: &sema::model::Declarations,
    name: &str,
) -> Result<(ItemId, TypeId)> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, d)| {
            let ty = facts.definitions.get(index).copied().flatten()?;
            if report.package.names.get(d.name) == name
                && let Some(Kind::Adt(item, _)) = facts.types.kind(ty)
            {
                Some((item, ty))
            } else {
                None
            }
        })
        .ok_or_else(|| format!("actual nominal {name}").into())
}
#[test]
fn actual_struct_enum_constructor_coordinates_preserve_strict_variant_queries() -> Result {
    let (report, facts) = facts()?;
    for name in ["N", "C", "E"] {
        let (item, ty) = nominal(&report, &facts, name)?;
        for (ordinal, &id) in facts.members.variants(item).iter().enumerate() {
            assert!(
                matches!(drops::nominal_position(&facts,ty,MemberQuery::Constructor(id))?,MemberPosition::Variant(actual) if actual as usize==ordinal)
            );
            let variant = drops::nominal_position(&facts, ty, MemberQuery::Variant(id));
            assert_eq!(variant.is_ok(), name == "E");
        }
    }
    Ok(())
}
#[test]
fn constructor_query_rejects_foreign_missing_stale_and_uninstalled_variants() -> Result {
    let (report, mut facts) = facts()?;
    let (c, ty) = nominal(&report, &facts, "C")?;
    let (e, _) = nominal(&report, &facts, "E")?;
    let id = *facts.members.variants(c).first().ok_or("C constructor")?;
    let foreign = *facts.members.variants(e).first().ok_or("E constructor")?;
    assert!(drops::nominal_position(&facts, ty, MemberQuery::Constructor(foreign)).is_err());
    assert!(
        drops::nominal_position(
            &facts,
            ty,
            MemberQuery::Constructor(sema::members::VariantId(u32::MAX))
        )
        .is_err()
    );
    let old = facts.members.variants[id.0 as usize];
    facts.members.variants[id.0 as usize].ordinal = 9;
    assert!(drops::nominal_position(&facts, ty, MemberQuery::Constructor(id)).is_err());
    facts.members.variants[id.0 as usize] = old;
    let extra = sema::members::VariantId(u32::try_from(facts.members.variants.len())?);
    facts.members.variants.push(old);
    assert!(drops::nominal_position(&facts, ty, MemberQuery::Constructor(extra)).is_err());
    Ok(())
}
#[test]
fn union_constructor_query_keeps_its_actual_non_field_destruction_policy() -> Result {
    let (report, facts) = facts()?;
    let (item, ty) = nominal(&report, &facts, "U")?;
    let id = *facts
        .members
        .variants(item)
        .first()
        .ok_or("actual union constructor")?;
    assert!(matches!(
        drops::nominal_position(&facts, ty, MemberQuery::Constructor(id))?,
        MemberPosition::Variant(0)
    ));
    assert_eq!(
        facts.aggregates.get(item).ok_or("union")?.kind,
        rustgm::types::aggregate::Kind::Union
    );
    let mut needs = rustgm::types::destruction::NeedsDrop::new(
        facts.destructors.implementations().iter().map(|d| d.item),
    );
    assert!(!needs.demand(&facts.types, &facts.aggregates, ty)?);
    Ok(())
}
