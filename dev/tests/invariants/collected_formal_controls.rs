//! Collection joins real written formals to canonical slots without wire guesses.
use super::*;
use crate::unpaid_cases::Result;
use std::fmt::Write as _;

const HOLDER: &str = "trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(value:&V)->&V{value}}impl Holder<u8> for u8{}";
fn actual(suffix: &str) -> Result<(Report, Declarations, Stage)> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    source.push_str(suffix);
    let report = crate::test_source::source_provider(&source, "formal_collection")?;
    let provider = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let stage = Stage::begin_with_ordinary_traits(&report, &mut analysis.facts, &provider)?;
    Ok((report, analysis.facts, stage))
}
fn named(report: &Report, name: &str) -> Result<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == DefKind::Function
                && report.package.names.get(definition.name) == name
        })
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or_else(|| "actual named function".into())
}
fn complete(suffix: &str) -> Result<(usize, usize)> {
    let (report, facts, mut stage) = actual(suffix)?;
    let expected = stage.selected_type_slots()?;
    let mut observed = 0usize;
    let mut implementations = 0usize;
    for index in 0..stage.definitions.len() {
        let Some(scope) = stage.collected_scope(&report, &facts, index)? else {
            continue;
        };
        observed +=
            stage.require_named_parameters(&report, &facts, scope.definition(), scope.source())?;
        implementations += usize::from(!scope.header());
    }
    assert_eq!(observed, expected);
    // Both real LegacyReceiver impl<T> scopes and the genuine Holder impl survive.
    assert!(implementations >= 3);
    let work = stage.work();
    let _completed = stage.collect_provider_headers(&report, &facts, |_, _| Ok(()))?;
    Ok((observed, work.steps))
}
#[test]
fn collected_formals_certify_full_family_impl_self_and_linear_slot_coverage() -> Result {
    let mut previous = None;
    for width in [1usize, 8, 32] {
        let mut source = format!("{HOLDER}fn unused<");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "T{index}")?;
        }
        source.push_str(">(value:u8)->u8{value}");
        let (slots, work) = complete(&source)?;
        if let Some((old_width, old_slots, old_work)) = previous {
            assert_eq!(slots - old_slots, width - old_width);
            assert!(work > old_work);
            assert!(work - old_work <= 200 * (width - old_width));
        }
        previous = Some((width, slots, work));
        eprintln!(
            "formal-collection width={width} slots={slots} work={work} collected_scope={}",
            size_of::<super::super::collected_provider::CollectedScope>()
        );
    }
    Ok(())
}
fn altered(opponent: usize) -> Result {
    let (report, mut facts, mut stage) =
        actual("fn selected<T,U>(value:T)->T{value}fn other<V>(value:V)->V{value}")?;
    let owner = named(&report, "selected")?;
    let other = named(&report, "other")?;
    let scope = stage
        .collected_scope(&report, &facts, owner.0 as usize)?
        .ok_or("actual selected scope")?;
    let foreign = facts
        .generics
        .parameters
        .get(&other)
        .and_then(|parameters| parameters.first())
        .copied()
        .ok_or("actual other formal")?;
    let raw_owner = stage.declaration(owner)?;
    match opponent {
        0 => {
            facts
                .generics
                .parameters
                .get_mut(&owner)
                .ok_or("actual formals")?
                .remove(0);
        }
        1 => {
            facts
                .generics
                .parameters
                .get_mut(&owner)
                .ok_or("actual formals")?
                .swap(0, 1);
        }
        2 => {
            facts
                .generics
                .parameters
                .get_mut(&owner)
                .ok_or("actual formals")?[0]
                .binding = foreign.binding;
        }
        3 => {
            facts
                .generics
                .parameters
                .get_mut(&owner)
                .ok_or("actual formals")?[0]
                .argument = foreign.argument;
        }
        4 => {
            stage
                .rows
                .slots
                .iter_mut()
                .find(|slot| slot.owner == raw_owner)
                .ok_or("actual raw slot")?
                .definition = None;
        }
        _ => {
            stage
                .rows
                .slots
                .iter_mut()
                .find(|slot| slot.owner == raw_owner)
                .ok_or("actual raw slot")?
                .ordinal += 1;
        }
    }
    let before = stage.work().steps;
    assert!(
        stage
            .require_named_parameters(&report, &facts, owner, scope.source())
            .is_err()
    );
    assert!(stage.work().steps > before);
    Ok(())
}
#[test]
fn collected_formals_reject_missing_reordered_foreign_and_anonymous_authority() -> Result {
    for opponent in 0..6 {
        altered(opponent)?;
    }
    let (report, mut facts, mut stage) = actual("fn anonymous(value:impl Clone){}")?;
    let owner = named(&report, "anonymous")?;
    let scope = stage
        .collected_scope(&report, &facts, owner.0 as usize)?
        .ok_or("actual anonymous owner")?;
    let parameter = facts
        .generics
        .parameters
        .get_mut(&owner)
        .and_then(|parameters| parameters.first_mut())
        .ok_or("actual anonymous parameter")?;
    assert!(parameter.anonymous);
    parameter.anonymous = false;
    let error = stage
        .require_named_parameters(&report, &facts, owner, scope.source())
        .err()
        .ok_or("wire-like false flag authorized anonymous input")?;
    assert!(
        error.message.contains("anonymous or missing named formal"),
        "{error}"
    );
    let (report, facts, mut stage) = actual("fn selected<T>(value:T)->T{value}")?;
    let owner = named(&report, "selected")?;
    let scope = stage
        .collected_scope(&report, &facts, owner.0 as usize)?
        .ok_or("actual selected owner")?;
    stage.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(
        stage
            .require_named_parameters(&report, &facts, owner, scope.source())
            .is_err()
    );
    assert!(stage.work().steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}
