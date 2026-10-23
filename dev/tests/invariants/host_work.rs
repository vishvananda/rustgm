//! Complete source paths and exact reconstruction work are measured separately.
use super::*;
use crate::types::instance::Instance;
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-payment/callable/work")
            .join(format!("{name}.rs")),
    )?)
}
fn observe(name: &str, calls: usize, expected_instances: usize) -> TestResult<usize> {
    let report = crate::test_source::source_provider(&fixture(name)?, "host_work")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let whole = facts.body_work.canonical_steps;
    let item = item(&report, &facts, "item")?;
    let Kind::FunctionItem {
        item: definition, ..
    } = facts.types.kind(item).ok_or("item")?
    else {
        return Err("item kind".into());
    };
    let arity = facts
        .generics
        .parameters
        .get(&crate::resolve::model::DefId(definition.0))
        .map_or(0, Vec::len);
    let selected = (0..facts.types.len())
        .map(|index| u32::try_from(index).map(TypeId))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|ty| matches!(facts.types.kind(*ty), Some(Kind::FunctionItem { item, arguments, .. }) if item == definition
            && facts.types.arguments(arguments).is_some_and(|args| args.len() == arity))
            && facts.types.contains_parameters(*ty) == Some(false)
            && facts.types.has_free_regions(*ty, 0) == Some(false))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), expected_instances, "{name}: {selected:?}");
    let mut context = Context::default();
    let before = facts.types.len();
    for index in 0..calls {
        let instance = Instance::new(&facts.types, selected[index % selected.len()])?;
        assert_eq!(
            validate_signature(&mut facts, &mut context, instance)?,
            None
        );
    }
    assert_eq!(
        facts.types.len(),
        before,
        "reconstruction retained duplicate types"
    );
    println!(
        "{name}: complete_lowering_work={whole} instances={} reconstructions={calls} reconstruction_work={} type_growth=0",
        selected.len(),
        context.steps
    );
    Ok(context.steps)
}
#[test]
fn complete_source_callable_axes_keep_structural_reconstruction_bounded() -> TestResult {
    for family in ["same-item", "distinct-instances"] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let work = observe(
                &format!("{family}-{width}"),
                width,
                if family == "same-item" { 1 } else { width },
            )?;
            if let Some(old) = previous {
                assert!(work <= old * 4 + 8);
            }
            previous = Some(work);
        }
    }
    let mut depth_work = None;
    for depth in [8, 32, 128] {
        let work = observe(&format!("shared-signature-{depth}-calls-8"), 8, 1)?;
        if let Some(old) = depth_work {
            assert_eq!(work, old, "closed signature subtree was rescanned");
        }
        depth_work = Some(work);
    }
    for calls in [32, 128] {
        let work = observe(&format!("shared-signature-32-calls-{calls}"), calls, 1)?;
        assert_eq!(work, depth_work.ok_or("depth work")? * (calls / 8));
    }
    Ok(())
}
#[test]
fn ordinary_pointer_goals_do_not_reconstruct_source_declarations() -> TestResult {
    for calls in [8, 32, 128] {
        let report = crate::test_source::source_provider(
            &fixture(&format!("ordinary-pointer-{calls}"))?,
            "ordinary_work",
        )?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let unused = item(&report, &facts, "unused")?;
        let Kind::FunctionItem { signature, .. } = facts.types.kind(unused).ok_or("unused")? else {
            return Err("unused kind".into());
        };
        let goal = predicate(&mut facts, signature, Constness::Ordinary)?;
        let mut context = Context::default();
        let mut fold = Instantiation::new(&mut context, &facts.types, &[])?;
        let goal = context.instantiate_predicate(&mut facts.types, goal, &mut fold)?;
        facts.function_indices.clear();
        let (before, types) = (context.steps, facts.types.len());
        for _ in 0..calls {
            assert!(matches!(
                for_goal(&mut facts, &mut context, &goal)?,
                Eligibility::Allowed
            ));
        }
        assert_eq!(context.steps, before);
        assert_eq!(facts.types.len(), types);
        println!("ordinary-pointer-{calls}: eligibility_work=0 type_growth=0");
    }
    Ok(())
}
