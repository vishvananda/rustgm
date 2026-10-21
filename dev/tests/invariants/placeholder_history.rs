//! Persistent placeholder histories survive assignment and shared canonical values.
use super::*;

#[test]
fn different_signature_origins_cannot_share_local_only_classes() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let (_, left) = definition(&mut store, &mut context)?;
    let (_, right) = definition(&mut store, &mut context)?;
    context.equate(&store, left[0], right[0])?;
    assert_eq!(context.slot_class(left[0])?, Class::Mixed);
    Ok(())
}

#[test]
fn assigned_local_slot_does_not_cross_into_an_unrelated_function_frame() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let (_, slots) = definition(&mut store, &mut context)?;
        assign(&mut context, &store, slots)?;
        let reference = local(&mut context, &store, 0)?;
        let unrelated = function(&mut context, &slots)?;
        let expected = function(&mut context, &[reference, reference])?;
        let (a, b) = if reverse {
            (expected, unrelated)
        } else {
            (unrelated, expected)
        };
        assert_eq!(
            context.equate_scoped_terms(&store, a, b),
            Err(Error::Mismatch)
        );
    }
    Ok(())
}

#[test]
fn nested_callable_depth_is_not_a_global_skip_rebase_flag() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let (signature, slots) = definition(&mut store, &mut context)?;
    let reference = local(&mut context, &store, 0)?;
    context.equate(&store, slots[0], reference)?;
    let nested = function(&mut context, &[slots[0]])?;
    // The original slot appears both directly and under another callable.
    // Under the nested frame it still denotes the enclosing signature slot.
    context.equate(&store, slots[1], nested)?;
    let outer_local = local(&mut context, &store, 1)?;
    let expected_nested = function(&mut context, &[outer_local])?;
    let expected = function(&mut context, &[reference, expected_nested])?;
    context.equate_scoped_terms(&store, signature, expected)?;
    assert!(context.region_equalities.is_empty());
    Ok(())
}

#[test]
fn registration_rejects_foreign_or_reused_slots_and_completed_owners() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let (signature, slots) = definition(&mut store, &mut context)?;
    let Some(owner) = context.signature_frame(signature) else {
        return Err("missing owner".into());
    };
    let environment = context.known(&store, TypeId::UNIT)?;
    let closure = context.closure(owner.origin, signature, environment, &[])?;
    assert_eq!(
        context.define_signature_slots(&store, closure, signature, &slots),
        Err(Error::InvalidIdentity)
    );
    store.complete_closure(owner.origin, crate::types::closure::CallKind::Shared)?;
    assert_eq!(
        context.define_signature_slots(&store, closure, signature, &slots),
        Err(Error::InvalidIdentity)
    );
    Ok(())
}

#[test]
fn an_actual_fold_never_revives_taint_on_an_existing_target() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let (_, slots) = definition(&mut store, &mut context)?;
    let before = local(&mut context, &store, 0)?;
    let after = local(&mut context, &store, 1)?;
    context.equate(&store, slots[0], before)?;
    let external = context.variable(Domain::Any)?;
    context.equate(&store, external, after)?;
    context.folded_slot_facts(before, after)?;
    assert_eq!(context.slot_class(external)?, Class::Mixed);
    Ok(())
}

#[test]
fn zero_slot_frame_and_depth_zero_use_preserve_ordinary_lifetime_meaning() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let slot = context.variable(Domain::Any)?;
    let signature = context.construct(
        Constructor::Function {
            bound_regions: 0,
            parameters: 0,
            unsafe_: false,
            convention: Convention::RUST,
        },
        &[slot],
    )?;
    let identity = store.reserve_closure()?;
    let environment = context.known(&store, TypeId::UNIT)?;
    let closure = context.closure(identity, signature, environment, &[])?;
    context.define_signature_slots(&store, closure, signature, &[slot])?;
    let value = local(&mut context, &store, 0)?;
    context.equate(&store, slot, value)?;
    context.equate_scoped_terms(&store, slot, value)?;
    let unrelated = context.construct(
        Constructor::Function {
            bound_regions: 0,
            parameters: 0,
            unsafe_: false,
            convention: Convention::RUST,
        },
        &[value],
    )?;
    assert_eq!(
        context.equate_scoped_terms(&store, signature, unrelated),
        Err(Error::Mismatch)
    );
    let outer = local(&mut context, &store, 1)?;
    let expected = context.construct(
        Constructor::Function {
            bound_regions: 0,
            parameters: 0,
            unsafe_: false,
            convention: Convention::RUST,
        },
        &[outer],
    )?;
    context.equate_scoped_terms(&store, signature, expected)?;
    assert!(context.signature_slots.is_none());
    Ok(())
}

#[test]
fn later_external_union_revokes_all_previously_assigned_equal_roots() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let (_, slots) = definition(&mut store, &mut context)?;
        let a = local(&mut context, &store, 0)?;
        let b = local(&mut context, &store, 0)?;
        context.equate(&store, slots[0], a)?;
        context.equate(&store, slots[1], b)?;
        context.equate(&store, slots[0], slots[1])?;
        let external = context.variable(Domain::Any)?;
        let (a, b) = if reverse {
            (external, slots[1])
        } else {
            (slots[1], external)
        };
        context.equate(&store, a, b)?;
        for slot in slots {
            assert_eq!(context.slot_class(slot)?, Class::Mixed);
        }
    }
    Ok(())
}

#[test]
fn importing_into_an_existing_unowned_value_never_relabels_it_as_owned() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (_, slots) = definition(&mut store, &mut source)?;
    let byte = source.known(&store, TypeId::primitive(Primitive::U8))?;
    source.equate(&store, slots[0], byte)?;
    let mut target = Context::default();
    let existing = target.known(&store, TypeId::primitive(Primitive::U8))?;
    target.import_slot_facts(&mut source, byte, existing, false)?;
    assert_eq!(target.slot_class(existing)?, Class::Empty);
    assert!(target.signature_slots.is_none());
    Ok(())
}

#[test]
fn assigned_history_union_work_is_bounded_without_old_table_rescans() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let mut context = Context::default();
        let slots = (0..=width)
            .map(|_| context.variable(Domain::Any))
            .collect::<Result<Vec<_>, _>>()?;
        let signature = function(&mut context, &slots)?;
        let environment = context.known(&store, TypeId::UNIT)?;
        let origin = store.reserve_closure()?;
        let closure = context.closure(origin, signature, environment, &[])?;
        context.define_signature_slots(&store, closure, signature, &slots)?;
        let start = context.steps;
        for &slot in &slots {
            let reference = local(&mut context, &store, 0)?;
            context.equate(&store, slot, reference)?;
        }
        for pair in slots.windows(2) {
            context.equate(&store, pair[0], pair[1])?;
        }
        let external = context.variable(Domain::Any)?;
        context.equate(&store, slots[width], external)?;
        assert_eq!(context.slot_class(slots[0])?, Class::Mixed);
        observations.push((width, context.steps - start));
    }
    println!("assigned histories: {observations:?}");
    let first = observations[1].1 - observations[0].1;
    let width = observations[1].0 - observations[0].0;
    for pair in observations.windows(2) {
        assert_eq!(
            (pair[1].1 - pair[0].1) * width,
            first * (pair[1].0 - pair[0].0)
        );
    }
    Ok(())
}

#[test]
fn canonical_sharing_is_not_union_but_explicit_assigned_equation_is() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let (a, left) = definition(&mut store, &mut context)?;
        let (b, right) = definition(&mut store, &mut context)?;
        let canonical = store.intern(crate::types::Kind::Reference {
            referent: TypeId::primitive(Primitive::U8),
            mutable: false,
            region: RegionId::bound(0, 0)?,
        })?;
        let reference = context.known(&store, canonical)?;
        for slots in [left, right] {
            context.equate(&store, slots[0], reference)?;
            let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
            context.equate(&store, slots[1], byte)?;
        }
        assert_eq!(context.root(left[0])?, context.root(right[0])?);
        let (a, b) = if reverse { (b, a) } else { (a, b) };
        context.equate_scoped_terms(&store, a, b)?;
        for slot in [left[0], right[0]] {
            let root = context.root(slot)?;
            assert!(
                context
                    .signature_definition(slot, root)?
                    .is_some_and(|(_, own)| own)
            );
        }
        context.equate(&store, left[0], right[0])?;
        for slot in [left[0], right[0]] {
            assert_eq!(context.slot_class(slot)?, Class::Mixed);
        }
    }
    Ok(())
}

#[test]
fn variable_birth_is_immutable_and_does_not_increase_term_size() -> TestResult {
    #[expect(
        dead_code,
        reason = "layout comparison against the pre-birth-marker term fields"
    )]
    struct Baseline {
        parent: TermId,
        rank: u8,
        value: Value,
        start: u32,
        end: u32,
    }
    type Term = super::super::super::Term;
    assert_eq!(std::mem::size_of::<Term>(), std::mem::size_of::<Baseline>());
    assert_eq!(
        std::mem::align_of::<Term>(),
        std::mem::align_of::<Baseline>()
    );
    let mut store = Store::default();
    let mut context = Context::default();
    let (_, slots) = definition(&mut store, &mut context)?;
    let external = context.variable(Domain::Integer)?;
    context.default_numbers()?;
    assert!(context.terms[external.0 as usize].was_variable);
    assert!(matches!(
        context.terms[external.0 as usize].value,
        Value::Known(_)
    ));
    context.equate(&store, slots[0], external)?;
    assert_eq!(context.slot_class(slots[0])?, Class::Mixed);
    println!("Term size {} bytes", std::mem::size_of::<Term>());
    Ok(())
}

#[test]
fn source_loss_refreshes_a_previously_imported_and_now_assigned_placeholder() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (_, slots) = definition(&mut store, &mut source)?;
    let (mut query, roots) = crate::types::infer::refinement::Refinement::new(
        &store,
        &mut source,
        &[Argument::Type(slots[0])],
    )?;
    let Argument::Type(copied) = roots[0] else {
        return Err("missing imported type".into());
    };
    let value = local(&mut source, &store, 0)?;
    source.equate(&store, slots[0], value)?;
    let external = source.variable(Domain::Any)?;
    source.equate(&store, slots[0], external)?;
    query.refresh(&store, &mut source)?;
    assert_eq!(query.context.slot_class(copied)?, Class::Mixed);
    Ok(())
}
