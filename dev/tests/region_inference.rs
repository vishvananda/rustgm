use rustgm::types::{
    ItemId, Kind, Primitive, RegionArgs, RegionId, Store, TypeId,
    infer::{Constructor, Context, Domain, Error},
};
#[path = "support/semantic.rs"]
mod semantic;

#[test]
fn unused_lifetime_obligations_cannot_bypass_the_executable_proof_gate()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let source = "fn required<'a:'static>(){}fn invalid<'a>(){required::<'a>();}fn main(){}";
    let (_, facts) = semantic::analyze(source)?;
    assert!(
        !facts.outlives.is_empty(),
        "the type view must preserve the pending bound"
    );
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/region-gate-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let input = directory.join("pending.rs");
    std::fs::write(&input, source)?;
    let rustc = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--emit=metadata",
            "--cap-lints=allow",
            "-o",
        ])
        .arg(directory.join("pending.rmeta"))
        .arg(&input)
        .output()?;
    assert_eq!(
        rustc.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&rustc.stderr)
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-lowir")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing compiler input")?
        .write_all(source.as_bytes())?;
    let result = child.wait_with_output()?;
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("completed region and loan proofs"));
    Ok(())
}

#[test]
fn already_materialized_closed_dags_share_work_without_sharing_fresh_positions() -> Result<(), Error>
{
    let mut store = Store::default();
    let mut cx = Context::default();
    let mut value = cx.known(&store, TypeId::UNIT)?;
    for _ in 0..100 {
        value = cx.construct(Constructor::Tuple, &[value, value])?;
    }
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            value,
            1
        )?,
        (value, 0)
    );
    assert!(
        cx.steps < 400,
        "closed DAG traversal must be linear in stored edges"
    );
    let leaf = cx.known(&store, TypeId::UNIT)?;
    let mut open = cx.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId::STATIC,
        },
        &[leaf],
    )?;
    for _ in 0..30 {
        open = cx.construct(Constructor::Tuple, &[open, open])?;
    }
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            open,
            1
        ),
        Err(Error::Budget),
        "distinct lifetime positions must charge before exponential expansion"
    );
    Ok(())
}

#[test]
fn local_assignments_keep_a_common_inferred_region_and_directed_sources() -> Result<(), String> {
    let (report, facts) = semantic::analyze("fn f<'a,'b>(x:&'a u8,y:&'b u8){let mut z=x;z=y;}")?;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == "z")
        .ok_or("missing local")?;
    let ty = facts.definitions[index].ok_or("missing local type")?;
    let Some(Kind::Reference { region, .. }) = facts.types.kind(ty) else {
        return Err("missing local reference".into());
    };
    assert!(region != RegionId::STATIC && region.bound_parts().is_none());
    assert!(
        facts.region_equalities.is_empty(),
        "subtyping must not equate independent inputs"
    );
    let mut sources = facts
        .coercions
        .iter()
        .filter(|c| c.expected == ty)
        .filter_map(|c| match facts.types.kind(c.actual) {
            Some(Kind::Reference { region, .. }) => Some(region),
            _ => None,
        })
        .collect::<Vec<_>>();
    sources.sort_unstable_by_key(|r| r.0);
    sources.dedup();
    assert_eq!(sources, [RegionId::bound(0, 0)?, RegionId::bound(0, 1)?]);
    Ok(())
}

#[test]
fn each_call_instantiates_late_lifetimes_once_and_retains_input_result_identity()
-> Result<(), String> {
    let (_, facts) = semantic::analyze(
        "fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8,y:&'b u8){let p=id;let _a=p(x);let _b=p(y);}",
    )?;
    let mut calls = Vec::new();
    for node in facts.nodes.iter().flatten() {
        let Some(rustgm::sema::model::Operation::Call { signature }) = node.operation else {
            continue;
        };
        let Some(Kind::Function {
            bound_regions,
            parameters,
            result,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("missing call signature".into());
        };
        assert_eq!(bound_regions, 0);
        assert_eq!(facts.types.list(parameters), Some([result].as_slice()));
        let Some(Kind::Reference { region, .. }) = facts.types.kind(result) else {
            return Err("missing call result reference".into());
        };
        calls.push(region);
    }
    assert_eq!(calls.len(), 2);
    assert_ne!(calls[0], calls[1]);
    assert_eq!(
        facts
            .body_regions
            .iter()
            .filter(|r| matches!(
                r.origin,
                rustgm::sema::model::BodyRegionOrigin::CallArgument
            ))
            .count(),
        2
    );
    Ok(())
}

#[test]
fn inferred_positions_are_independent_without_defaulting_numeric_referents()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let number = cx.variable(Domain::Integer)?;
    let source = cx.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId::STATIC,
        },
        &[number],
    )?;
    let tuple = cx.construct(Constructor::Tuple, &[source, source])?;
    let (fresh, count) = cx.fresh_regions(
        &mut store,
        &rustgm::types::callable::Templates::default(),
        tuple,
        10,
    )?;
    assert_eq!(count, 2);
    assert_eq!(cx.resolved(&mut store, number), Err(Error::Unresolved));
    for (index, region) in [RegionId(10), RegionId(11)].into_iter().enumerate() {
        let field = cx.child(&store, fresh, index)?;
        assert_eq!(
            cx.head(&store, field)?,
            Some(Constructor::Reference {
                mutable: false,
                region
            })
        );
        assert_eq!(cx.child(&store, field, 0)?, number);
    }
    let expected = cx.known(&store, TypeId::primitive(Primitive::U16))?;
    cx.equate(&store, number, expected)?;
    assert!(cx.region_equalities.is_empty());
    cx.finish(&mut store)?;
    Ok(())
}

#[test]
fn arrays_nominal_parameters_and_function_item_identity_keep_their_structure()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let arguments = store.intern_regions(&[RegionId::STATIC, RegionId::STATIC])?;
    let adt = store.intern(Kind::Adt(ItemId(3), arguments))?;
    let array = store.intern(Kind::Array {
        element: adt,
        length: 10_000,
    })?;
    let mut cx = Context::default();
    let array = cx.known(&store, array)?;
    let (array, count) = cx.fresh_regions(
        &mut store,
        &rustgm::types::callable::Templates::default(),
        array,
        1,
    )?;
    assert_eq!(
        count, 2,
        "one type per array element, not per runtime value"
    );
    let element = cx.child(&store, array, 0)?;
    let Some(Constructor::Adt(item, arguments)) = cx.head(&store, element)? else {
        return Err("missing nominal type".into());
    };
    assert_eq!(item, ItemId(3));
    assert_eq!(
        store.region_args(arguments),
        Some([RegionId(1), RegionId(2)].as_slice())
    );
    let parameters = store.intern_list(&[])?;
    let signature = store.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 0,
        unsafe_: false,
    })?;
    let item = store.intern(Kind::FunctionItem {
        item,
        arguments: RegionArgs::EMPTY,
        signature,
    })?;
    let item = cx.known(&store, item)?;
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            item,
            3
        )?,
        (item, 0)
    );
    Ok(())
}

#[test]
fn callable_binders_preserve_bound_lifetimes_and_freshen_only_free_positions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let late = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::UNIT,
    })?;
    let captured = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: TypeId::UNIT,
    })?;
    let parameters = store.intern_list(&[late, captured])?;
    let signature = store.intern(Kind::Function {
        parameters,
        result: late,
        bound_regions: 1,
        unsafe_: false,
    })?;
    assert_eq!(store.contains_regions(signature), Some(true));
    assert_eq!(store.has_free_regions(signature, 0), Some(true));
    assert_eq!(store.has_free_regions(signature, 1), Some(false));
    let mut cx = Context::default();
    let original = cx.known(&store, signature)?;
    let (fresh, count) = cx.fresh_regions(
        &mut store,
        &rustgm::types::callable::Templates::default(),
        original,
        5,
    )?;
    assert_eq!(count, 1);
    for position in [0, 2] {
        let child = cx.child(&store, fresh, position)?;
        assert_eq!(cx.resolved(&mut store, child)?, late);
    }
    let captured = cx.child(&store, fresh, 1)?;
    assert_eq!(
        cx.head(&store, captured)?,
        Some(Constructor::Reference {
            mutable: false,
            region: RegionId(5)
        })
    );
    let parameters = store.intern_list(&[late])?;
    let closed = store.intern(Kind::Function {
        parameters,
        result: late,
        bound_regions: 1,
        unsafe_: false,
    })?;
    assert_eq!(store.has_free_regions(closed, 0), Some(false));
    let closed = cx.known(&store, closed)?;
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            closed,
            6
        )?,
        (closed, 0)
    );
    Ok(())
}

#[test]
fn deep_freshening_and_closed_dags_are_bounded_and_iterative()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut cx = Context::default();
    let mut deep = cx.known(&store, TypeId::UNIT)?;
    for _ in 0..10_000 {
        deep = cx.construct(
            Constructor::Reference {
                mutable: false,
                region: RegionId::STATIC,
            },
            &[deep],
        )?;
    }
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            deep,
            1
        )?
        .1,
        10_000
    );
    assert!(cx.steps < 25_000);
    let mut ty = TypeId::UNIT;
    for _ in 0..1_000 {
        let children = store.intern_list(&[ty, ty])?;
        ty = store.intern(Kind::Tuple(children))?;
    }
    let known = cx.known(&store, ty)?;
    let before = cx.steps;
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            known,
            10_001
        )?,
        (known, 0)
    );
    assert_eq!(cx.steps, before);
    let leaf = cx.known(&store, TypeId::UNIT)?;
    let reference = cx.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId::STATIC,
        },
        &[leaf],
    )?;
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            reference,
            0
        ),
        Err(Error::Overflow)
    );
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            reference,
            0x8000_0000
        ),
        Err(Error::Overflow)
    );
    cx.steps = 1_000_000;
    assert_eq!(
        cx.fresh_regions(
            &mut store,
            &rustgm::types::callable::Templates::default(),
            reference,
            1
        ),
        Err(Error::Budget)
    );
    let closed = store.intern(Kind::Adt(ItemId(4), RegionArgs::EMPTY))?;
    assert_eq!(store.contains_regions(closed), Some(false));
    Ok(())
}

#[test]
fn item_templates_preserve_formal_identity_even_when_original_arguments_are_equal()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, facts) = semantic::analyze(
        "fn g<'a:'a,'b:'b>(_:&'a u8,x:&'b u8)->&'b u8{x}fn f(){let p=g::<'static,'static>;}",
    )?;
    let converted = facts
        .coercions
        .iter()
        .find_map(|c| match facts.types.kind(c.expected) {
            Some(Kind::FunctionItem {
                item,
                arguments,
                signature,
            }) => Some((item, arguments, signature)),
            _ => None,
        })
        .ok_or("missing inferred function item")?;
    let (item, args, signature) = converted;
    let arguments = facts
        .types
        .region_args(args)
        .ok_or("missing inferred arguments")?;
    assert_eq!(arguments.len(), 2);
    assert_ne!(arguments[0], arguments[1]);
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("missing instantiated callable".into());
    };
    assert_eq!(bound_regions, 0);
    let inputs = facts
        .types
        .list(parameters)
        .ok_or("missing callable inputs")?;
    for (ty, expected) in [
        (inputs[0], arguments[0]),
        (inputs[1], arguments[1]),
        (result, arguments[1]),
    ] {
        assert!(
            matches!(facts.types.kind(ty), Some(Kind::Reference { region, .. }) if region==expected)
        );
    }
    let template = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("missing original template")?;
    assert_eq!(template.parameters, 2);
    assert_eq!(facts.types.escaping_binders(template.signature), Some(1));
    Ok(())
}

#[test]
fn fresh_callable_types_retain_their_declarations_well_formedness_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::outlives::{Bounds, Queries};
    let (_, facts) =
        semantic::analyze("fn g<'a:'static>()->&'a u8{loop{}}fn f(){let p=g::<'static>;}")?;
    let converted = facts
        .coercions
        .iter()
        .find(|c| {
            matches!(
                facts.types.kind(c.expected),
                Some(Kind::FunctionItem { .. })
            )
        })
        .ok_or("missing inferred item conversion")?;
    let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(converted.expected) else {
        return Err("missing inferred item type".into());
    };
    let arguments = facts
        .types
        .region_args(arguments)
        .ok_or("missing inferred arguments")?;
    assert_eq!(arguments.len(), 1);
    assert_ne!(arguments[0], RegionId::STATIC);
    let bounds = Bounds::infer(&facts.types, &facts.aggregates, &[])?;
    let mut queries = Queries::new(&facts.types, &bounds, &facts.lifetimes.callables);
    assert_eq!(
        queries.requirements(converted.expected)?,
        &[(arguments[0], RegionId::STATIC)]
    );
    Ok(())
}

#[test]
fn nested_callable_specialization_is_bounded_and_keeps_positions_independent()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let reference = store.intern(Kind::Reference {
        region: RegionId::bound(0, 0)?,
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    let parameters = store.intern_list(&[reference])?;
    let poly = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: reference,
        unsafe_: false,
    })?;
    let list = store.intern_list(&[poly, poly])?;
    let pair = store.intern(Kind::Tuple(list))?;
    let specialized = rustgm::types::specialize::callables(&mut store, pair, 1)?;
    assert_eq!(specialized.regions, 2);
    let Some(Kind::Tuple(list)) = store.kind(specialized.ty) else {
        return Err("missing specialized pair".into());
    };
    let functions = store.list(list).ok_or("missing functions")?;
    assert_ne!(functions[0], functions[1]);
    for (index, ty) in functions.iter().enumerate() {
        let Some(Kind::Function {
            bound_regions,
            parameters,
            result,
            ..
        }) = store.kind(*ty)
        else {
            return Err("missing specialized function".into());
        };
        assert_eq!(bound_regions, 0);
        assert_eq!(store.list(parameters), Some([result].as_slice()));
        let Some(Kind::Reference { region, .. }) = store.kind(result) else {
            return Err("missing specialized reference".into());
        };
        assert_eq!(region, RegionId(u32::try_from(index + 1)?));
    }
    let mut deep = poly;
    for _ in 0..4_000 {
        let list = store.intern_list(&[deep])?;
        deep = store.intern(Kind::Tuple(list))?;
    }
    let specialized = rustgm::types::specialize::callables(&mut store, deep, 7)?;
    assert_eq!(specialized.regions, 1);
    assert!(specialized.steps < 9_000);
    assert_eq!(store.contains_late_callables(specialized.ty), Some(false));
    let mut closed = TypeId::UNIT;
    for _ in 0..100 {
        let list = store.intern_list(&[closed, closed])?;
        closed = store.intern(Kind::Tuple(list))?;
    }
    let specialized = rustgm::types::specialize::callables(&mut store, closed, 7)?;
    assert_eq!(specialized.ty, closed);
    assert_eq!(specialized.regions, 0);
    assert_eq!(specialized.steps, 1);
    assert!(rustgm::types::specialize::callables(&mut store, poly, u32::MAX).is_err());
    Ok(())
}
