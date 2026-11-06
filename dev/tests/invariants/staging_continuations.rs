//! Staging rows are paid syntax and retain no permission after the owner drain.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const _: [(); 96] = [(); size_of::<super::super::Pending>()];
const _: [(); 80] = [(); size_of::<Binding>()];
const _: [(); 56] = [(); size_of::<Syntax>()];
const _: [(); 96] = [(); size_of::<Pending>()];
const _: [(); 32] = [(); size_of::<Owner>()];
const _: [(); 32] = [(); size_of::<Prepared>()];
const _: [(); 80] = [(); size_of::<super::super::nest::Run>()];
const _: [(); 104] = [(); super::super::nest::task_bytes()];
fn owner(report: &crate::resolve::expansion::Report, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .ok_or("actual owner")?;
    Ok(DefId(u32::try_from(index)?))
}
#[test]
fn equality_only_and_ordinary_headers_do_not_allocate_projected_staging() -> TestResult {
    let report =
        crate::test_source::resolve("trait A{type Out;}fn f<T:A<Out=u8>>(){}fn main(){}", true)?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    assert!(checker.associated_bindings.projected.is_none());
    Ok(())
}
#[test]
fn failed_prepaid_growth_is_sticky_and_cannot_publish_a_stage() -> TestResult {
    let report = crate::test_source::resolve(
        "trait A{type Out;}trait B{}fn f<T:A<Out:B>>(){}fn main(){}",
        true,
    )?;
    let mut checker = Checker::new(&report)?;
    assert!(checker.charge_bound_staging(1, usize::MAX).is_err());
    let first = crate::sema::header_attributes::source_contract_costs(&checker.facts);
    assert!(
        checker
            .prepare_projected_owner(owner(&report, "f")?)
            .is_err()
    );
    let second = crate::sema::header_attributes::source_contract_costs(&checker.facts);
    assert_eq!(first.1, usize::MAX);
    assert!(second.0 > first.0);
    assert_eq!(second.1, first.1);
    assert!(checker.associated_bindings.projected.is_none());
    Ok(())
}
#[test]
fn late_header_roots_cannot_replay_after_the_real_requirement_drain() -> TestResult {
    let text = crate::test_source::fixture("pa20/tests/spec/assoc-bounds/ldebt.rs")?;
    let report = crate::test_source::resolve(&text, true)?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let owner = owner(&report, "X")?;
    let stage = checker
        .associated_bindings
        .projected
        .as_ref()
        .ok_or("real stage")?;
    let ((_, binding), prepared) = stage
        .interpretations
        .iter()
        .find(|((_, b), _)| b.owner == owner)
        .ok_or("real child interpretation")?;
    let bound = *prepared.syntax.first().ok_or("actual written bound")?;
    let context = bound.context(binding.context);
    let (path, definition) = bound.trait_path().ok_or("actual trait argument path")?;
    let (arguments, _) = checker.supplied_arguments(path, definition)?;
    let node = *report
        .package
        .unit(path.file)
        .configured
        .ast
        .list(arguments.ok_or("actual argument list")?)
        .first()
        .ok_or("actual projection argument")?;
    let mut roots = vec![(NodeKey { node, ..path }, context)];
    let error = checker
        .stage_projected_roots(owner, &mut roots)
        .err()
        .ok_or("consumed roots were accepted")?;
    assert!(error.message.contains("completed owner phase"), "{error}");
    assert_eq!(roots.len(), 1);
    assert!(
        checker
            .stage_projected_roots(DefId(u32::MAX), &mut roots)
            .is_err()
    );
    Ok(())
}

#[test]
fn a_real_pending_child_retains_its_prefix_and_stable_subject() -> TestResult {
    let text = crate::test_source::fixture("pa20/tests/spec/assoc-bounds/nested.rs")?;
    let report = crate::test_source::resolve(&text, true)?;
    let mut checker = Checker::new(&report)?;
    checker.run()?;
    let definition = owner(&report, "f")?;
    let key = *checker
        .associated_bindings
        .projected
        .as_ref()
        .ok_or("actual cache")?
        .interpretations
        .keys()
        .find(|(_, b)| {
            b.owner == definition
                && matches!(
                    checker.facts.types.kind(b.predicate.ty),
                    Some(crate::types::Kind::Parameter(_))
                )
        })
        .ok_or("actual outer interpretation")?;
    let super::super::constraint::Outcome::Bounds(subject) =
        checker.complete_binding_constraint(key.0, key.1)?
    else {
        return Err("actual projected subject".into());
    };
    checker
        .associated_bindings
        .projected
        .as_mut()
        .ok_or("actual cache")?
        .interpretations
        .clear();
    let mut output = Vec::new();
    let mut edges = Vec::new();
    assert!(checker.complete_projected_bounds(key.0, key.1, subject, &mut output, &mut edges)?);
    assert!(output.is_empty());
    let mut steps = 0;
    let limit = checker.work_limit();
    checker.drain(&mut steps, limit)?;
    let super::super::constraint::Outcome::Bounds(current) =
        checker.complete_binding_constraint(key.0, key.1)?
    else {
        return Err("actual resumed subject".into());
    };
    assert_eq!(subject, current);
    assert!(checker.complete_projected_bounds(key.0, key.1, current, &mut output, &mut edges)?);
    assert!(output.is_empty());
    let run = checker
        .associated_bindings
        .projected
        .as_ref()
        .and_then(|p| p.runs.get(&key))
        .ok_or("actual suspended producer")?;
    assert_eq!(run.prefix_len(), 1);
    checker.drain(&mut steps, limit)?;
    assert!(!checker.complete_projected_bounds(key.0, key.1, current, &mut output, &mut edges)?);
    assert_eq!(output.len(), 2);
    assert!(
        checker
            .associated_bindings
            .projected
            .as_ref()
            .is_some_and(|p| p.runs.is_empty())
    );
    Ok(())
}
