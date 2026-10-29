//! Primitive constant members use real per-owner conformance before executable selection.
use super::*;
const TRAIT: &str = "pub const trait Limits{const LOW:u8;const HIGH:u8;fn value()->usize;}";
const IMPL: &str = "const impl Limits for u8{const LOW:u8=3;const HIGH:u8=9;fn value()->usize{7}}";
fn input(tail: &str) -> Result<String> {
    Ok(format!("{}\n{TRAIT}\n{tail}", source("runtime-read")?))
}
fn checked_report(source: &str) -> Result<crate::resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        source,
        "constant_conformance",
    )?)
}
#[test]
fn real_primitive_constants_and_method_complete_before_annotation_execution() -> Result {
    let text = input(&format!("{IMPL}type Called=[u8;<u8 as Limits>::value()];"))?;
    let report = checked_report(&text)?;
    let mut checker = Checker::new(&report)?;
    checker
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    checker.run()?;
    let owner = *owners(&checker, "Limits")?.first().ok_or("actual impl")?;
    assert!(
        checker
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(owner))
    );
    let mut pending = Pending::default();
    let mut plan = plan(&mut checker, &mut pending, owner)?;
    assert!(plan.has_constants());
    assert!(plan.require_no_associated(&checker.facts, owner).is_err());
    plan.require_constants(&checker.facts, owner)?;
    let member = plan.members.pop().ok_or("actual member")?;
    assert!(plan.require_constants(&checker.facts, owner).is_err());
    plan.members.push(member);
    plan.require_constants(&checker.facts, owner)?;
    let mut foreign = Checker::new(&report)?;
    foreign
        .facts
        .set_constant_executor(crate::pipeline::execution::Context::default().executor());
    foreign.run()?;
    assert!(plan.require_constants(&foreign.facts, owner).is_err());
    Ok(())
}
#[test]
fn wrong_missing_extra_and_ordinary_members_do_not_grant_const_execution() -> Result {
    for (name, changed) in [
        ("wrong", IMPL.replace("const HIGH:u8=9", "const HIGH:u16=9")),
        ("missing", IMPL.replace("const HIGH:u8=9;", "")),
        (
            "extra",
            IMPL.replace("fn value()", "const FOREIGN:u8=7;fn value()"),
        ),
        ("ordinary", IMPL.replace("const impl", "impl")),
    ] {
        let text = input(&format!(
            "{changed}type Called=[u8;<u8 as Limits>::value()];"
        ))?;
        let report = checked_report(&text)?;
        let mut checker = Checker::new(&report)?;
        checker
            .facts
            .set_constant_executor(crate::pipeline::execution::Context::default().executor());
        let error = checker
            .run()
            .err()
            .ok_or("invalid impl unexpectedly accepted")?;
        assert!(
            !error.message.contains("constant member index requires"),
            "{name}: {error}"
        );
        assert!(
            !error.message.contains("outside the early scalar profile"),
            "{name}: {error}"
        );
        if name == "ordinary" {
            assert!(
                error.message.contains(
                    "trait obligation requires an unambiguous matching assumption or implementation"
                ),
                "{error}"
            );
            let owner = *owners(&checker, "Limits")?
                .first()
                .ok_or("actual ordinary impl")?;
            assert!(matches!(
                checker
                    .facts
                    .associated
                    .owner(owner)
                    .and_then(|id| checker.facts.associated.get(id))
                    .map(|owner| owner.kind),
                Some(crate::sema::associated::OwnerKind::Impl {
                    constant: false,
                    ..
                })
            ));
        }
        if name != "ordinary" {
            for owner in owners(&checker, "Limits")? {
                assert!(
                    !checker
                        .facts
                        .generics
                        .implementations
                        .as_ref()
                        .is_some_and(|r| r.is_validated(owner)),
                    "{name}"
                );
            }
        }
    }
    Ok(())
}
#[test]
fn constant_owner_indexes_keep_warm_storage_and_failed_growth_bounded() -> Result {
    let report = checked_report(&input(IMPL)?)?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = *owners(&checker, "Limits")?.first().ok_or("actual owner")?;
    let mut members = super::super::super::constants::Members::default();
    members.prepare_source_owner(&report, &mut checker.facts, owner)?;
    let before = members.indexed_storage();
    for _ in 0..128 {
        members.prepare_source_owner(&report, &mut checker.facts, owner)?;
    }
    assert_eq!(members.indexed_storage(), before);
    let foreign = crate::test_source::source_provider(&input(IMPL)?, "foreign_constant_owner")?;
    assert!(
        members
            .prepare_source_owner(&foreign, &mut checker.facts, owner)
            .is_err()
    );
    let mut fresh = super::super::super::constants::Members::default();
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(
            &checker.facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    assert!(
        fresh
            .prepare_source_owner(&report, &mut checker.facts, owner)
            .is_err()
    );
    assert_eq!(fresh.indexed_storage(), (0, 0));
    // The failed prepayment must not make even an empty owner bucket visible.
    assert!(
        fresh
            .selected(&mut checker.facts, &report, owner, owner)
            .is_err()
    );
    assert_eq!(std::mem::size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn constant_member_width_growth_is_local_to_the_actual_owner_bucket() -> Result {
    use std::fmt::Write as _;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut text = source("runtime-read")?;
        text.push_str("trait Wide:Sized{");
        for n in 0..width {
            write!(text, "const K{n}:u8;")?;
        }
        text.push_str("}impl Wide for u8{");
        for n in 0..width {
            write!(text, "const K{n}:u8=7;")?;
        }
        text.push('}');
        let report = checked_report(&text)?;
        let mut checker = Checker::new(&report)?;
        checker.run()?;
        let owner = *owners(&checker, "Wide")?.first().ok_or("actual Wide")?;
        let mut members = super::super::super::constants::Members::default();
        members.prepare_source_owner(&report, &mut checker.facts, owner)?;
        let (count, bytes) = members.indexed_storage();
        assert_eq!(count, 1);
        if let Some(old) = previous {
            assert!(bytes <= old * 5);
        }
        previous = Some(bytes);
    }
    Ok(())
}
