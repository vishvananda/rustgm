//! A closure's signature and captured arguments retain one nominal lifetime identity.
use rustgm::types::{
    Kind, RegionId, Store, TypeId,
    arguments::Argument,
    callable::{Convention, Templates},
    closure::CallKind,
    infer::Context,
};

#[test]
fn inferred_bindings_preserve_coupled_closure_regions() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let reference = types.intern(Kind::Reference {
        mutable: false,
        region: RegionId(7),
        referent: TypeId::UNIT,
    })?;
    let parameters = types.intern_list(&[reference])?;
    let signature = types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: reference,
        unsafe_: false,
        convention: Convention::RUST,
    })?;
    let identity = types.declare_closure(CallKind::Shared)?;
    let arguments = types.intern_arguments(&[Argument::Lifetime(RegionId(7))])?;
    let closure = types.intern(Kind::Closure {
        identity,
        arguments,
        signature,
        environment: TypeId::UNIT,
    })?;
    let mut inference = Context::default();
    let original = inference.known(&types, closure)?;
    for materialized in [false, true] {
        if materialized {
            inference.head(&types, original)?;
        }
        let before = inference.steps;
        let fresh = inference.fresh_regions(&mut types, &Templates::default(), original, 10)?;
        assert_eq!(fresh.term, original);
        assert_eq!(fresh.regions, 0);
        assert!(!fresh.needs_normalization);
        assert!(inference.steps - before <= 2);
        assert_eq!(inference.resolved(&mut types, fresh.term)?, closure);
    }
    Ok(())
}
