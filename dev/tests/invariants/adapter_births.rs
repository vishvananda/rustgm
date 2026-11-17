//! Selected receiver adapters retain births and transfers for owned call values.
use super::{
    TestResult, body, build_adapter, fixture_with_report, originals, proved_sources, selected,
};
use crate::{
    body::ir::{Owner, Rvalue, Terminator, ValueId},
    support::call_traits::CallTrait,
    types::{closure::CallKind, destruction::NeedsDrop},
};

fn verify(source: &str, expected_owned: usize) -> TestResult {
    let (report, mut fixture) = fixture_with_report(source)?;
    let facts = &mut fixture.facts;
    let checked = originals(facts, &fixture.templates, &fixture.inputs)?;
    let entry = selected(
        facts,
        &checked,
        *fixture.instances.first().ok_or("instance")?,
    )?;
    let adapter = entry.adapter().ok_or("selected receiver adapter")?;
    assert_eq!(adapter.requested(), CallTrait::FnMut);
    let (_, execution) = entry.validate_adapter(facts)?;
    assert_eq!(execution.canonical.receiver, CallKind::Shared);
    assert_eq!(execution.execution.receiver, CallKind::Mutable);
    let (mut builder, original, _) =
        proved_sources(&report, facts, &checked, fixture.program, &fixture.inputs)?;
    let prepared = build_adapter(facts, entry, builder.body(original)?)?;
    assert!(matches!(prepared.body.owner, Owner::ClosureAdapter { .. }));
    let mut query = NeedsDrop::new(facts.destructors.implementations().iter().map(|d| d.item));
    let needed = prepared
        .body
        .values
        .iter()
        .map(|value| query.demand(&facts.types, &facts.aggregates, value.ty))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        needed.iter().filter(|&&owned| owned).count(),
        expected_owned
    );
    for (ordinal, &needed) in needed.iter().enumerate() {
        let value = ValueId(u32::try_from(ordinal)?);
        let recorded = prepared
            .body
            .owned
            .as_ref()
            .and_then(|owned| owned.creation(value));
        assert_eq!(
            recorded.is_some(),
            needed,
            "selected adapter value {ordinal}"
        );
    }
    let Terminator::Return(result) = prepared
        .body
        .blocks
        .last()
        .ok_or("adapter block")?
        .terminator
    else {
        return Err("adapter return".into());
    };
    assert!(matches!(
        prepared.body.values[result.0 as usize].kind,
        Rvalue::Invoke { .. }
    ));
    if expected_owned == 0 {
        assert!(prepared.body.owned.is_none());
    } else {
        let owned = prepared.body.owned.as_ref().ok_or("owned adapter values")?;
        assert_eq!(owned.created.len(), expected_owned);
        assert_eq!(owned.returned.len(), 1);
        assert_eq!(owned.returned[0].value, result);
        let exit = prepared
            .body
            .storage
            .as_ref()
            .and_then(|s| s.exits.first())
            .ok_or("adapter exit")?;
        assert_eq!(owned.returned[0].location, exit.location);
        assert_eq!(owned.returned[0].exits_before, 0);
    }
    body::drops::requirements(facts, &prepared.body, &mut query)?;
    let mut budget = 0;
    let index = builder.push_adapter(&report, facts, &checked, original, prepared, &mut budget)?;
    assert!(std::ptr::eq(
        builder
            .caller_body(&facts.types, index)?
            .ok_or("source bridge")?,
        builder.body(original)?,
    ));
    if expected_owned != 0 {
        let mut erased = build_adapter(facts, entry, builder.body(original)?)?;
        erased.body.owned = None;
        let error = builder
            .push_adapter(&report, facts, &checked, original, erased, &mut budget)
            .err()
            .ok_or("erased owned coverage was accepted")?;
        assert_eq!(error.message, "owned value lacks its destruction scope");
    }
    Ok(())
}

#[test]
fn copy_outputs_remain_unowned_in_selected_adapters() -> TestResult {
    verify(
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<F:FnMut()->u8>(mut f:F)->u8{f()}fn main(){let f=||7u8;check(call(f)==7);}
",
        0,
    )
}

#[test]
fn owned_outputs_have_birth_and_return_transfer_in_selected_adapters() -> TestResult {
    verify(
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D(u8);impl Drop for D{fn drop(&mut self){}}fn call<F:FnMut()->D>(mut f:F)->D{f()}fn main(){let f=||D(7);let result=call(f);check(result.0==7);}
",
        1,
    )
}

#[test]
fn owned_inputs_and_outputs_keep_distinct_transfers_in_selected_adapters() -> TestResult {
    verify(
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}fn call<'a,F:FnMut(D<'a>)->D<'a>>(mut f:F,d:D<'a>)->D<'a>{f(d)}fn main(){let mut n=0u8;{let f=|d|d;let result=call(f,D(&mut n));check(*result.0==0);}check(n==1);}
",
        2,
    )
}

fn factory_fixture(
    width: usize,
) -> TestResult<(
    crate::sema::model::Declarations,
    body::ir::Body,
    body::ir::Body,
)> {
    let foundation =
        crate::test_source::fixture("pa31/tests/spec/needs-drop/work/width8-calls8.rs")?;
    let foundation = foundation
        .split("pub mod intrinsics")
        .next()
        .ok_or("foundation")?;
    let fields = vec!["Leaf"; width].join(",");
    let source = format!(
        "{foundation}struct Leaf;impl Drop for Leaf{{fn drop(&mut self){{}}}}\
         struct Root({fields});fn make()->Root{{Root({fields})}}\
         fn generic<T>(value:T)->T{{value}}fn main(){{}}"
    );
    let report = crate::test_source::source_provider(&source, "factory_cleanup_work")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let program = body::build_concrete(&report, &facts)?;
    let factory = program
        .bodies
        .into_iter()
        .find(|body| {
            body.locals.len() == width
                && body.values.len() == width + 1
                && body.blocks.len() == 1
                && matches!(
                    body.values.last().map(|v| v.kind),
                    Some(Rvalue::Aggregate { .. })
                )
        })
        .ok_or("actual tuple constructor")?;
    let generic = body::templates::build(&report, &facts)?
        .bodies
        .into_iter()
        .find(|template| facts.types.contains_parameters(template.body.result) == Some(true))
        .ok_or("actual generic input body")?
        .body;
    facts.open_lowering_account(&report)?;
    Ok((facts, factory, generic))
}

#[test]
fn factory_cleanup_shares_queries_and_pays_bounded_work_on_success_and_failure() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let (mut facts, factory, mut generic) = factory_fixture(width)?;
        let mut cleanup = body::instantiate::Cleanup::new(&facts);
        let mut cold = factory.clone();
        cold.owned = None;
        let before = cleanup.steps();
        let before_paid = facts.body_work.canonical_steps;
        let (result, header_work) =
            facts.pay_demand_traversal(|facts| cleanup.factory_values(facts, &mut cold));
        let cold_work = facts.body_work.canonical_steps - before_paid;
        assert_eq!(header_work, [0, 0]);
        result?;
        let query_work = cleanup.steps() - before;
        assert!(query_work > 0);
        assert_eq!(
            cold.owned.as_ref().ok_or("factory births")?.created.len(),
            width + 1
        );
        let mut warm = factory;
        warm.owned = None;
        let before_paid = facts.body_work.canonical_steps;
        let (result, header_work) =
            facts.pay_demand_traversal(|facts| cleanup.factory_values(facts, &mut warm));
        let warm_work = facts.body_work.canonical_steps - before_paid;
        assert_eq!(header_work, [0, 0]);
        result?;
        assert_eq!(cleanup.steps() - before, query_work);
        assert_eq!(cold_work - warm_work, query_work);
        assert!(warm_work > 0 && warm_work < 64 * width + 256);
        if let Some(smaller) = previous {
            assert!(warm_work <= 4 * smaller + 128);
        }
        previous = Some(warm_work);
        generic.owned = None;
        let before = cleanup.steps();
        let before_paid = facts.body_work.canonical_steps;
        let (result, header_work) =
            facts.pay_demand_traversal(|facts| cleanup.factory_values(facts, &mut generic));
        let paid = facts.body_work.canonical_steps - before_paid;
        assert_eq!(header_work, [0, 0]);
        assert_eq!(
            result,
            Err("generic destruction checking requires a parameter environment")
        );
        assert!(cleanup.steps() > before);
        assert!(paid > cleanup.steps() - before);
    }
    Ok(())
}
