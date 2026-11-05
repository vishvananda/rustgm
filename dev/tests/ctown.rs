//! Actual const Drop declarations validate every field before any cleanup demand.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> Result<(String, rustgm::resolve::expansion::Report)> {
    let text = source::fixture(&format!("pa31/tests/spec/source-const-cleanup/{name}.rs"))?;
    let report = source::source_provider(&text, "const_drop_owner")?;
    Ok((text, report))
}
#[test]
fn unused_const_owners_preserve_real_conditional_lifetime_union_and_manual_policies() -> Result {
    for name in [
        "owncnd", "cpok", "ownref", "uncond", "owngud", "ownman", "tagow2",
    ] {
        let (_, report) = input(name)?;
        pipeline::analyze_source(&report).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn unused_invalid_const_owners_require_every_real_declared_field_const_premise() -> Result {
    for name in [
        "owncp", "ownar0", "ownar1", "ownunc", "uncopy", "ownbad", "tagown",
    ] {
        let (text, report) = input(name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or_else(|| format!("{name}: invalid unused owner accepted"))?;
        assert!(
            error.message.contains("const Drop declaration field")
                || error.message.contains("can't drop"),
            "{name}: {error}"
        );
        assert!(
            error.file.source().is_some() && error.span.start > 0,
            "{name}: unlocated {error}"
        );
        let nominal = text
            .rfind("pub struct S")
            .or_else(|| text.rfind("pub union U"))
            .or_else(|| text.rfind("pub enum E"))
            .ok_or("nominal source")?;
        assert!(
            error.span.start as usize > nominal,
            "{name}: moved away from actual impl: {error}"
        );
    }
    Ok(())
}
#[test]
fn whole_bounds_and_unused_owner_validation_keep_their_actual_distinct_causes() -> Result {
    for name in ["whoar0", "whocp"] {
        let (_, report) = input(name)?;
        let error = pipeline::analyze_source(&report)
            .err()
            .ok_or("invalid owner accepted")?;
        assert!(
            error.message.contains("const Drop declaration field")
                || error.message.contains("can't drop"),
            "{name}: {error}"
        );
    }
    let (_, report) = input("whon0")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("whole array field bound waived")?;
    assert!(error.message.contains("can't drop"), "{error}");
    Ok(())
}
#[test]
fn repeated_semantic_fields_have_bounded_shared_fold_work() -> Result {
    let mut work = Vec::new();
    for name in ["ownw0", "ownw8", "ownw32", "own128"] {
        let (_, report) = input(name)?;
        work.push(
            pipeline::analyze_source(&report)?
                .facts
                .body_work
                .canonical_steps,
        );
    }
    let baseline = work[0];
    let growth = work[1..]
        .iter()
        .map(|&n| n.checked_sub(baseline).ok_or("work decreased"))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    eprintln!("const Drop field work: base={baseline}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
    Ok(())
}
