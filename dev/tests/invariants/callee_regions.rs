//! Captured regions retain exact identities across the empty intrinsic binder.
use super::instance_types_match;
use crate::types::{Kind, Primitive, RegionId, Store, TypeId, callable::Convention};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[test]
fn opened_call_shape_keeps_exact_flags_arity_and_caller_coordinates() -> TestResult {
    let mut store = Store::default();
    let input = reference(&mut store, 0, 1)?;
    let shifted = reference(&mut store, 1, 1)?;
    let parameters = store.intern_list(&[input])?;
    let public_parameters = store.intern_list(&[shifted])?;
    let public = store.intern(Kind::Function {
        bound_regions: 0,
        parameters: public_parameters,
        result: shifted,
        unsafe_: true,
        convention: Convention::RUST,
    })?;
    let selected = super::TransmuteSelection {
        definition: crate::resolve::model::DefId(0),
        signature: public,
        source: input,
        destination: input,
    };
    let actual = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: input,
        unsafe_: true,
        convention: Convention::RUST,
    })?;
    assert!(selected.opened_signature_matches(&store, actual));
    assert!(!selected.opened_signature_matches(&store, public));
    for (bound_regions, unsafe_, convention, result) in [
        (1, true, Convention::RUST, input),
        (0, false, Convention::RUST, input),
        (0, true, Convention::C, input),
        (0, true, Convention::RUST, TypeId::UNIT),
    ] {
        let wrong = store.intern(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        })?;
        assert!(!selected.opened_signature_matches(&store, wrong));
    }
    let parameters = store.intern_list(&[input, input])?;
    let wrong = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: input,
        unsafe_: true,
        convention: Convention::RUST,
    })?;
    assert!(!selected.opened_signature_matches(&store, wrong));
    Ok(())
}
#[test]
fn constant_borrowed_elements_keep_the_actual_reached_intrinsic_call_contract() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-transmute/generic-borrowed-const.rs")?;
    let report = crate::test_source::source_provider(&source, "borrowed_const")?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, d)| {
            if report.package.names.get(d.name) != "LENGTH" {
                return None;
            }
            Some(crate::resolve::model::DefId(u32::try_from(index).ok()?))
        })
        .ok_or("actual source LENGTH")?;
    let reader = crate::sema::values::Reader::new(&facts);
    let value = reader
        .definition(definition)
        .ok_or("source LENGTH was not evaluated")?;
    assert!(matches!(
        reader.view(value)?,
        crate::sema::values::View::Scalar(crate::sema::model::Scalar::Integer(1))
    ));
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "length"
        })
        .copied()
        .ok_or("actual borrowed const method")?;
    let built = prepared.templates.work().0;
    let template = prepared.templates.function(&report, &facts, &function)?;
    assert_eq!(
        prepared.templates.work().0,
        built,
        "rebuild of retained CFG"
    );
    assert_eq!(template.body.instance, None);
    assert!(template.body.const_context);
    assert!(template.body.promotions.is_some());
    prepared.templates.retain(template)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/transmute-regions-v1/candidate-const");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn actual_borrowed_slice_transmute_preserves_the_complete_generic_source_accessor() -> TestResult {
    for name in ["generic-borrowed-slice", "generic-borrowed-method"] {
        let source =
            crate::test_source::fixture(&format!("pa31/tests/spec/source-transmute/{name}.rs"))?;
        let report = crate::test_source::source_provider(&source, "borrowed_transmute")?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .map_err(|error| format!("{name}: {error}"))?;
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == "observed" && s.body.is_some())
                .count(),
            1,
            "{name}"
        );
    }
    Ok(())
}
fn reference(store: &mut Store, depth: u16, slot: u16) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        referent: TypeId::UNIT,
        region: RegionId::bound(depth, slot)?,
    })
}
fn callable(store: &mut Store, local: TypeId, captured: TypeId) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(&[local, captured])?;
    store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: captured,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
#[test]
fn nested_binders_keep_local_slots_and_reject_wrong_capture_type_and_unshifted_roots() -> TestResult
{
    let mut store = Store::default();
    let local = reference(&mut store, 0, 0)?;
    let captured = reference(&mut store, 1, 1)?;
    let shifted = reference(&mut store, 2, 1)?;
    let original = callable(&mut store, local, captured)?;
    let signature = callable(&mut store, local, shifted)?;
    let inputs = store.intern_list(&[signature])?;
    let count = store.len();
    let mut paid = 0;
    assert!(instance_types_match(
        &store,
        inputs,
        signature,
        [original, original],
        |n| {
            paid += n;
            Ok(())
        }
    )?);
    assert!(paid > 1);
    assert_eq!(store.len(), count);
    let wrong = reference(&mut store, 1, 2)?;
    let wrong = callable(&mut store, local, wrong)?;
    assert!(!instance_types_match(
        &store,
        inputs,
        signature,
        [wrong, original],
        |_| Ok(())
    )?);
    assert!(!instance_types_match(
        &store,
        inputs,
        signature,
        [TypeId::primitive(Primitive::Usize), original],
        |_| Ok(())
    )?);
    // Same raw IDs are insufficient when an empty signature binder captures slot0.
    let inputs = store.intern_list(&[local])?;
    assert!(!instance_types_match(
        &store,
        inputs,
        local,
        [local, local],
        |_| Ok(())
    )?);
    let before = store.len();
    assert!(!instance_types_match(
        &store,
        inputs,
        local,
        [local, local],
        |_| Ok(())
    )?);
    assert_eq!(store.len(), before);
    Ok(())
}
#[test]
fn rejected_lookup_and_payment_keep_attempted_work_and_publish_no_type() -> TestResult {
    let mut store = Store::default();
    let original = reference(&mut store, 0, 1)?;
    let shifted = reference(&mut store, 1, 1)?;
    let inputs = store.intern_list(&[shifted])?;
    let before = store.len();
    for limit in [0, 1] {
        let mut paid = 0;
        let result = instance_types_match(&store, inputs, shifted, [original, original], |n| {
            paid += n;
            if paid > limit {
                Err("fixture source query payment exhausted")
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        assert!(paid > limit);
        assert_eq!(store.len(), before);
    }
    let mut paid = 0;
    assert!(
        instance_types_match(
            &store,
            inputs,
            TypeId(u32::MAX),
            [original, original],
            |n| {
                paid += n;
                Ok(())
            }
        )
        .is_err()
    );
    assert!(paid > 1);
    assert_eq!(store.len(), before);
    Ok(())
}
#[test]
fn shared_captured_roots_are_bounded_without_interning_or_cached_call_permission() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut store = Store::default();
        let mut original = reference(&mut store, 0, 1)?;
        let mut shifted = reference(&mut store, 1, 1)?;
        for _ in 0..width {
            let children = store.intern_list(&[original, original])?;
            original = store.intern(Kind::Tuple(children))?;
            let children = store.intern_list(&[shifted, shifted])?;
            shifted = store.intern(Kind::Tuple(children))?;
        }
        let inputs = store.intern_list(&[shifted])?;
        let count = store.len();
        let mut cold = 0;
        assert!(instance_types_match(
            &store,
            inputs,
            shifted,
            [original, original],
            |n| {
                cold += n;
                Ok(())
            }
        )?);
        let mut warm = 0;
        assert!(instance_types_match(
            &store,
            inputs,
            shifted,
            [original, original],
            |n| {
                warm += n;
                Ok(())
            }
        )?);
        assert_eq!(cold, warm);
        assert!(cold <= width * 8 + 16);
        if let Some(before) = previous {
            assert!(cold <= before * 5);
        }
        previous = Some(cold);
        assert_eq!(store.len(), count);
    }
    Ok(())
}
