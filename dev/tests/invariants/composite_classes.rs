//! Placed class summaries retain exact layout identity and bounded shared-graph work.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn facts(source: &str) -> TestResult<crate::sema::model::Declarations> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}
fn signature(facts: &crate::sema::model::Declarations) -> TestResult<TypeId> {
    facts.functions.iter().find_map(|f| matches!(facts.types.kind(f.signature),Some(Kind::Function { convention, .. }) if convention.abi==types::callable::Abi::C).then_some(f.signature)).ok_or_else(|| "C signature".into())
}
#[test]
fn shifted_classes_preserve_public_records_and_flat_path_allocation() -> TestResult {
    assert_eq!(size_of::<Plans>(), 88);
    assert_eq!(size_of::<Passing>(), 16);
    assert_eq!(size_of::<Object>(), 16);
    assert_eq!(size_of::<Plan>(), 40);
    let mut flat = facts("#[repr(C)]struct V{a:u64,b:f64}extern \"C\" fn f(x:V)->V{x}fn main(){}")?;
    let sig = signature(&flat)?;
    let mut plans = Plans::default();
    plans.demand(&flat.types, &flat.aggregates, &mut flat.layouts, sig)?;
    assert!(
        plans.composites.is_none(),
        "flat scalar fields allocate no composite cache"
    );
    let source = std::fs::read_to_string(format!(
        "{}/../pa30/tests/spec/aggregate-abi/composites/shifted-nested-mixed.rs",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let mut nested = facts(&source)?;
    let sig = signature(&nested)?;
    let mut plans = Plans::default();
    let plan = plans.demand(&nested.types, &nested.aggregates, &mut nested.layouts, sig)?;
    assert_eq!(
        plan.result,
        Passing::Object(Object {
            bytes: 16,
            align: 4,
            classes: Classes::Eightbytes {
                first: Class::Integer,
                second: Some(Class::Sse)
            }
        })
    );
    assert!(plans.composites.is_some());
    let retained = plans.retained_bytes;
    let steps = plans.steps;
    for _ in 0..128 {
        assert!(Arc::ptr_eq(
            &plan,
            &plans.demand(&nested.types, &nested.aggregates, &mut nested.layouts, sig)?
        ));
    }
    assert_eq!(plans.steps - steps, 128);
    assert_eq!(plans.retained_bytes, retained);
    eprintln!(
        "ABI storage: Plans={} Passing={} Object={} Plan={}",
        size_of::<Plans>(),
        size_of::<Passing>(),
        size_of::<Object>(),
        size_of::<Plan>()
    );
    Ok(())
}
#[test]
fn memory_objects_still_validate_nested_representations() -> TestResult {
    for inner in [
        "struct I{a:u32}",
        "#[repr(C,packed)]struct I{a:u8,b:u32}",
        "#[repr(C)]union I{a:u32,b:f32}",
    ] {
        let source = format!(
            "{inner}#[repr(C)]struct V{{wide:[u64;3],inner:I}}extern \"C\" fn f(x:V)->V{{x}}fn main(){{}}"
        );
        let mut facts = facts(&source)?;
        let signature = signature(&facts)?;
        let result = Plans::default().demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            signature,
        );
        assert!(
            result.is_err(),
            "MEMORY cannot hide unsupported fields: {inner}"
        );
    }
    Ok(())
}

fn graph_source(depth: usize, roots: usize, unrelated: usize) -> TestResult<String> {
    use std::fmt::Write;
    let mut source = String::from("#[repr(C)]struct N0{a:u32,b:f32}");
    for index in 1..=depth {
        write!(source, "#[repr(C)]struct N{index}{{inner:N{}}}", index - 1)?;
    }
    for index in 0..roots {
        write!(
            source,
            "#[repr(C)]struct V{index}{{head:u32,inner:N{depth},tail:f32}}extern \"C\" fn f{index}(x:V{index})->V{index}{{x}}"
        )?;
    }
    for index in 0..unrelated {
        write!(
            source,
            "struct Unrelated{index};fn unused{index}(_:Unrelated{index}){{}}"
        )?;
    }
    source.push_str("fn main(){}");
    Ok(source)
}
fn observe(source: &str) -> TestResult<(usize, usize, usize, usize)> {
    let mut facts = facts(source)?;
    let signatures=facts.functions.iter().filter_map(|function| matches!(facts.types.kind(function.signature),Some(Kind::Function {convention,..}) if convention.abi==types::callable::Abi::C).then_some(function.signature)).collect::<Vec<_>>();
    let mut plans = Plans::default();
    for signature in signatures {
        plans.demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            signature,
        )?;
    }
    let cache = plans.composites.as_ref().ok_or("composite cache")?;
    let (nodes, classes) = cache.counts();
    assert!(classes <= 8 * nodes);
    Ok((plans.steps, plans.retained_bytes, nodes, classes))
}
#[test]
fn composite_dag_and_placement_reuse_scale_with_new_nodes() -> TestResult {
    let mut depths = Vec::new();
    let mut roots = Vec::new();
    let mut unrelated = Vec::new();
    let mut arrays = Vec::new();
    for width in [8, 32, 128] {
        depths.push(observe(&graph_source(width, 1, 0)?)?);
        roots.push(observe(&graph_source(8, width, 0)?)?);
        unrelated.push(observe(&graph_source(8, 1, width)?)?);
        arrays.push(observe(&format!(
            "#[repr(C)]struct V{{a:[u32;{width}]}}extern \"C\" fn f(x:V)->V{{x}}fn main(){{}}"
        ))?);
        eprintln!(
            "composites width={width} depth={:?} roots={:?} unrelated={:?} memory_array={:?}",
            depths.last(),
            roots.last(),
            unrelated.last(),
            arrays.last()
        );
    }
    for values in [&depths, &roots] {
        assert_eq!(values[2].0 - values[1].0, 4 * (values[1].0 - values[0].0));
        assert_eq!(values[2].1 - values[1].1, 4 * (values[1].1 - values[0].1));
    }
    assert_eq!(unrelated[0], unrelated[1]);
    assert_eq!(unrelated[1], unrelated[2]);
    assert_eq!(arrays[0], arrays[1]);
    assert_eq!(arrays[1], arrays[2]);
    Ok(())
}

#[test]
fn actual_argument_classification_reuses_shared_graphs_without_growing_signatures() -> TestResult {
    for depth in [8, 32, 128] {
        let mut facts = facts(&graph_source(depth, 1, 0)?)?;
        let signature = signature(&facts)?;
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
            return Err("C function signature".into());
        };
        let argument = *facts
            .types
            .list(parameters)
            .and_then(|values| values.first())
            .ok_or("C argument type")?;
        let mut plans = Plans::default();
        let expected = plans.passing(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            argument,
        )?;
        let retained = plans.retained_bytes;
        let counts = plans.composites.as_ref().ok_or("composite cache")?.counts();
        for repeats in [8, 32, 128] {
            let before = plans.steps;
            for _ in 0..repeats {
                assert_eq!(
                    plans.passing(
                        &facts.types,
                        &facts.aggregates,
                        &mut facts.layouts,
                        argument
                    )?,
                    expected
                );
            }
            assert_eq!(plans.steps - before, repeats);
            assert_eq!(plans.retained_bytes, retained);
            assert_eq!(
                plans.composites.as_ref().ok_or("composite cache")?.counts(),
                counts
            );
            assert!(plans.signatures.is_empty());
            assert_eq!(plans.types.len(), 1);
            eprintln!(
                "actual ABI depth={depth} repeats={repeats} steps={repeats} retained={retained}"
            );
        }
    }
    Ok(())
}
