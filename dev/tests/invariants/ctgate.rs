//! Data completeness and Code ownership never substitute for actual const proof.
use super::*;
use crate::{
    body,
    const_eval::code::{Code, Prepared},
    sema, test_source,
};
type Result<T = ()> = std::result::Result<T, String>;
pub(in crate::const_eval) fn inert() -> Result<(Declarations, Body, Rc<Bundle>)> {
    let report = test_source::resolve("const fn f(){}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let body = program.bodies.into_iter().next().ok_or("body")?;
    let contract = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|c| c.key == body.key())
        .ok_or("contract")?;
    let types = facts.types.arena_identity();
    let definitions = facts.aggregates.arena_identity();
    let bundle = Rc::new(Bundle {
        view: View {
            events: Vec::new(),
            recipes: body::drops::glue::Recipes::default(),
            calls: Vec::new(),
            active: Vec::new(),
            cleanup_count: 0,
            blocks: Vec::new(),
            edges: crate::support::hash::HashMap::default(),
        },
        contract: Arc::new(contract),
        key: body.key(),
        source: body.source,
        signatures: body.owner.signatures(),
        result: body.result,
        types,
        definitions,
        binding: RefCell::new(None),
    });
    Ok((facts, body, bundle))
}
#[test]
fn bound_code_rejects_duplicate_foreign_and_expired_owners() -> Result {
    let (facts, body, bundle) = inert()?;
    let code = Rc::new(Code::snapshot(&body)?);
    bundle.bind(&code)?;
    let mut exhausted = crate::body::defining_cleanups::Work::new(0);
    for _ in 0..2 {
        assert_eq!(
            bundle.reached(&facts, &code, 0, &mut exhausted).err(),
            Some("defining cleanup work budget exceeded")
        );
    }
    assert!(exhausted.steps >= 64);
    assert!(bundle.require_unbound().is_err());
    assert!(bundle.bind(&code).is_err());
    let mut other = Code::snapshot(&body)?;
    let mut work = crate::body::defining_cleanups::Work::new(128);
    assert!(
        bundle
            .reached(&facts, &other, 0, &mut work)
            .err()
            .is_some_and(|e| e.contains("another actual Code"))
    );
    other
        .operands
        .push(crate::const_eval::code::PreparedOperand::Cleanup(
            Rc::clone(&bundle),
        ));
    drop(code);
    let other = Rc::new(other);
    assert!(bundle.bind(&other).is_err());
    assert!(
        bundle
            .reached(&facts, &other, 0, &mut work)
            .err()
            .is_some_and(|e| e.contains("live Code owner"))
    );
    Ok(())
}
#[test]
fn footer_projection_prepays_and_refuses_duplicate_or_bound_payloads() -> Result {
    let (_, body, bundle) = inert()?;
    let mut work = 0;
    let mut prepared = Prepared::plain(&body, usize::MAX)?;
    assert!(
        prepared
            .attach_cleanup(Rc::clone(&bundle), 0, &mut work)
            .is_err()
    );
    assert!(prepared.code().cleanup().is_none());
    prepared.attach_cleanup(Rc::clone(&bundle), usize::MAX, &mut work)?;
    assert!(
        prepared
            .attach_cleanup(Rc::clone(&bundle), usize::MAX, &mut work)
            .is_err()
    );
    assert!(work > 0);
    let code = Rc::new(Code::snapshot(&body)?);
    bundle.bind(&code)?;
    let mut fresh = Prepared::plain(&body, usize::MAX)?;
    assert!(fresh.attach_cleanup(bundle, usize::MAX, &mut work).is_err());
    assert!(fresh.code().cleanup().is_none());
    Ok(())
}
#[test]
fn sparse_call_inventory_distinguishes_removed_active_and_added_inactive_rows() -> Result {
    let (facts, body, bundle) = inert()?;
    let body::ir::Owner::Function { definition, .. } = body.owner else {
        return Err("function".into());
    };
    let ty = facts.definitions[definition.0 as usize].ok_or("function item")?;
    let instance = crate::types::instance::Instance::new(&facts.types, ty)?;
    let row = Method {
        cleanup: 1,
        place: body::ir::Place::Local(body::ir::LocalId(0)),
        target: TypeId::UNIT,
        instance,
        recipe: body::drops::glue::Id(0),
    };
    let mut view = Rc::try_unwrap(bundle).map_err(|_| "shared")?.view;
    view.cleanup_count = 3;
    view.active = vec![2];
    view.calls = vec![row];
    view.validate()?;
    assert!(view.call(0)?.is_none());
    assert!(view.call(1)?.is_some());
    view.calls.clear();
    assert!(view.validate().is_err());
    assert!(view.call(1).is_err());
    view.calls.push(row);
    view.calls.push(Method { cleanup: 2, ..row });
    assert!(view.validate().is_err());
    assert!(view.call(2).is_err());
    view.calls.pop();
    view.calls.push(row);
    assert!(view.validate().is_err());
    view.calls.pop();
    view.active[0] |= 8;
    assert!(view.validate().is_err());
    assert!(view.call(3).is_err());
    Ok(())
}
#[test]
fn ordinary_widths_and_cold_payloads_are_explicit() {
    assert_eq!(size_of::<crate::const_eval::Frame>(), 248);
    assert_eq!(size_of::<crate::const_eval::ReturnTo>(), 8);
    assert_eq!(size_of::<Code>(), 304);
    assert_eq!(size_of::<crate::const_eval::code::PreparedOperand>(), 32);
    assert_eq!(size_of::<Bundle>(), 328);
    assert_eq!(size_of::<View>(), 240);
    assert_eq!(size_of::<Method>(), 24);
}
