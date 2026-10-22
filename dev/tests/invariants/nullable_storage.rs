//! Nullable Reader witnesses reject wrong owners, ordinals, counts and logical fields.
use super::*;
use crate::{sema::values::Evaluator, types::ItemId};
#[test]
fn nullable_reader_checks_exact_member_owner_count_and_field_identity() -> Result<(), String> {
    let report = crate::test_source::resolve(
        "fn pass(v:Option<&'static u32>)->Option<&'static u32>{v}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "pass")
        .ok_or("function")?;
    let Some(Kind::FunctionItem { signature, .. }) =
        facts.types.kind(facts.definitions[index].ok_or("type")?)
    else {
        return Err("item".into());
    };
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("signature".into());
    };
    let ty = *facts
        .types
        .list(parameters)
        .and_then(|p| p.first())
        .ok_or("parameter")?;
    Evaluator::new(&report, &mut facts)
        .storage_layout(ty, 10000)
        .map_err(|e| e.message().to_owned())?;
    let stored = Reader::new(&facts).enum_niche(ty)?;
    let (empty, payload) = (stored.empty, stored.payload);
    let original = facts.members.variants[payload.0 as usize];
    assert!(Reader::new(&facts).niche_projection(ty, 1).is_err());
    facts.members.variants[payload.0 as usize].owner = ItemId(u32::MAX);
    assert!(Reader::new(&facts).enum_niche(ty).is_err());
    facts.members.variants[payload.0 as usize] = original;
    facts.members.variants[payload.0 as usize].ordinal =
        facts.members.variants[empty.0 as usize].ordinal;
    assert!(Reader::new(&facts).enum_niche(ty).is_err());
    facts.members.variants[payload.0 as usize] = original;
    facts.members.variants[payload.0 as usize].end = original.start;
    assert!(Reader::new(&facts).enum_niche(ty).is_err());
    facts.members.variants[payload.0 as usize] = original;
    assert!(Reader::new(&facts).enum_niche(ty).is_ok());
    assert_eq!(size_of::<crate::sema::values::StoredShape<'_>>(), 32);
    Ok(())
}
