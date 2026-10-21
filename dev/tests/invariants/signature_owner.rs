//! Signature authority follows definitions, never matching callable shape alone.
use super::*;
use crate::types::{Primitive, RegionId, TypeId, callable::Convention, infer::arguments::Argument};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn function(context: &mut Context, children: &[TermId]) -> Result<TermId, Error> {
    context.construct(
        Constructor::Function {
            bound_regions: 1,
            parameters: u32::try_from(children.len() - 1).map_err(|_| Error::Overflow)?,
            unsafe_: false,
            convention: Convention::RUST,
        },
        children,
    )
}
fn definition(
    store: &mut Store,
    context: &mut Context,
) -> Result<(TermId, [TermId; 2]), Box<dyn std::error::Error>> {
    let slots = [
        context.variable(Domain::Any)?,
        context.variable(Domain::Any)?,
    ];
    let signature = function(context, &slots)?;
    let environment = context.known(store, TypeId::UNIT)?;
    let identity = store.reserve_closure()?;
    let closure = context.closure(identity, signature, environment, &[])?;
    context.define_signature_slots(store, closure, signature, &slots)?;
    Ok((signature, slots))
}
fn local(
    context: &mut Context,
    store: &Store,
    depth: u16,
) -> Result<TermId, Box<dyn std::error::Error>> {
    let byte = context.known(store, TypeId::primitive(Primitive::U8))?;
    Ok(context.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId::bound(depth, 0)?,
        },
        &[byte],
    )?)
}
fn assign(context: &mut Context, store: &Store, slots: [TermId; 2]) -> TestResult {
    let value = local(context, store, 0)?;
    context.equate(store, slots[0], value)?;
    let byte = context.known(store, TypeId::primitive(Primitive::U8))?;
    context.equate(store, slots[1], byte)?;
    Ok(())
}

#[test]
fn assigned_signature_slots_compare_in_their_actual_frame_in_both_orders() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let (signature, slots) = definition(&mut store, &mut context)?;
        assign(&mut context, &store, slots)?;
        let reference = local(&mut context, &store, 0)?;
        let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
        let expected = function(&mut context, &[reference, byte])?;
        let (a, b) = if reverse {
            (expected, signature)
        } else {
            (signature, expected)
        };
        context.equate_scoped_terms(&store, a, b)?;
        assert!(context.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn same_owner_unions_keep_definition_but_external_unions_lose_it_monotonically() -> TestResult {
    for reverse in [false, true] {
        for assigned in [false, true] {
            let mut store = Store::default();
            let mut context = Context::default();
            let (_, slots) = definition(&mut store, &mut context)?;
            context.equate(&store, slots[0], slots[1])?;
            let root = context.root(slots[0])?;
            assert!(
                context
                    .signature_definition(slots[1], root)?
                    .is_some_and(|(_, own)| own)
            );
            if assigned {
                let reference = local(&mut context, &store, 0)?;
                context.equate(&store, root, reference)?;
            }
            let external = context.variable(Domain::Any)?;
            let (a, b) = if reverse {
                (external, slots[0])
            } else {
                (slots[0], external)
            };
            context.equate(&store, a, b)?;
            let root = context.root(slots[0])?;
            assert_eq!(context.slot_class(slots[0])?, Class::Mixed);
            assert!(
                context
                    .signature_definition(slots[0], root)?
                    .is_some_and(|(_, own)| !own)
            );
        }
    }
    Ok(())
}

#[test]
fn persistent_equality_of_previously_assigned_roots_revokes_local_only_authority() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let (_, slots) = definition(&mut store, &mut context)?;
        let external = context.variable(Domain::Any)?;
        let a = local(&mut context, &store, 0)?;
        let b = local(&mut context, &store, 0)?;
        context.equate(&store, slots[0], a)?;
        context.equate(&store, external, b)?;
        let (a, b) = if reverse {
            (external, slots[0])
        } else {
            (slots[0], external)
        };
        context.equate(&store, a, b)?;
        assert_eq!(context.slot_class(slots[0])?, Class::Mixed);
    }
    Ok(())
}

#[test]
fn imports_cannot_grant_closed_frame_authority_or_revive_mixed_history() -> TestResult {
    use super::super::import::Import;
    let mut store = Store::default();
    let mut source = Context::default();
    let (signature, slots) = definition(&mut store, &mut source)?;
    let mut query = Context::default();
    let mut mapping = Import::default();
    let copied = query.import_with(&store, &mut source, signature, &mut mapping)?;
    assert!(
        query
            .signature_frame(copied)
            .is_some_and(|owner| !owner.closed)
    );
    let hole = query.child(&store, copied, 0)?;
    let external = query.variable(Domain::Any)?;
    query.equate(&store, hole, external)?;
    assert_eq!(
        query.import_with(&store, &mut source, signature, &mut mapping)?,
        copied
    );
    assert_eq!(query.slot_class(hole)?, Class::Mixed);
    source.receive_slot_loss(slots[0], &mut query, hole)?;
    assert_eq!(source.slot_class(slots[0])?, Class::Mixed);
    Ok(())
}

#[test]
fn actual_binder_fold_retains_frame_origin_but_shape_alike_function_does_not() -> TestResult {
    let mut store = Store::default();
    let mut context = Context::default();
    let (signature, slots) = definition(&mut store, &mut context)?;
    assign(&mut context, &store, slots)?;
    let mapped = context.map_regions(
        &mut store,
        signature,
        &mut crate::types::substitute::Bound::shift(1),
    )?;
    assert_eq!(
        context.signature_frame(mapped),
        context.signature_frame(signature)
    );
    let other = function(&mut context, &slots)?;
    assert_eq!(context.signature_frame(other), None);
    Ok(())
}

#[test]
fn unresolved_nested_capture_stays_strict_in_both_operand_orders() -> TestResult {
    for reverse in [false, true] {
        let mut store = Store::default();
        let mut context = Context::default();
        let (signature, _) = definition(&mut store, &mut context)?;
        let reference = local(&mut context, &store, 0)?;
        let expected = function(&mut context, &[reference, reference])?;
        let (a, b) = if reverse {
            (expected, signature)
        } else {
            (signature, expected)
        };
        assert_eq!(
            context.equate_scoped_terms(&store, a, b),
            Err(Error::Mismatch)
        );
    }
    Ok(())
}

#[test]
fn refinement_round_trips_propagate_loss_before_source_recommit() -> TestResult {
    let mut store = Store::default();
    let mut source = Context::default();
    let (_, slots) = definition(&mut store, &mut source)?;
    let (mut query, roots) = super::super::refinement::Refinement::new(
        &store,
        &mut source,
        &[Argument::Type(slots[0])],
    )?;
    let Argument::Type(copied) = roots[0] else {
        return Err("missing type root".into());
    };
    let external = query.context.variable(Domain::Any)?;
    query.context.equate(&store, copied, external)?;
    for _ in 0..3 {
        query.outgoing_regions(
            &mut store,
            &mut source,
            &mut crate::types::substitute::Bound::new(Vec::new()),
        )?;
        query.refresh(&store, &mut source)?;
        assert_eq!(source.slot_class(slots[0])?, Class::Mixed);
    }
    Ok(())
}

#[test]
fn ordinary_inference_allocates_no_signature_catalog() -> TestResult {
    let store = Store::default();
    let mut context = Context::default();
    let a = context.variable(Domain::Any)?;
    let b = context.known(&store, TypeId::UNIT)?;
    context.equate(&store, a, b)?;
    assert!(context.signature_slots.is_none());
    Ok(())
}

#[path = "placeholder_history.rs"]
mod history;
