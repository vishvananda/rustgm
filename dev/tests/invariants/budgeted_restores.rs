//! Fixed-map budgets cover ordinary roots, nested binders and wrapper scratch.
use super::*;
use crate::types::{Kind, Primitive};
fn byte() -> TypeId {
    TypeId::primitive(Primitive::U8)
}
fn reference(store: &mut Store, region: RegionId) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: byte(),
    })
}
fn folder(bytes: usize, work: usize) -> Result<Bound, &'static str> {
    Bound::restore_budgeted_regions(
        HashMap::from([
            (RegionId(1), RegionId(20)),
            (RegionId::bound(0, 0)?, RegionId::bound(0, 1)?),
        ]),
        work,
        bytes,
    )
}
#[test]
fn restore_budget_preserves_layout_and_never_retains_unknown_ids() -> Result<(), &'static str> {
    assert_eq!(size_of::<Bound>(), 208);
    assert_eq!(size_of::<Mapping>(), size_of::<PreviousMapping>());
    eprintln!(
        "restore-layout bound={} mapping={} previous-mapping={}",
        size_of::<Bound>(),
        size_of::<Mapping>(),
        size_of::<PreviousMapping>()
    );
    let mut store = Store::default();
    let mut fold = folder(1 << 20, 1000)?;
    assert!(fold.region(RegionId(2), 0).is_err());
    assert!(fold.region(RegionId(20), 0).is_err());
    assert_eq!(fold.region(RegionId(1), 0)?, RegionId(20));
    assert_eq!(
        fold.region(RegionId::bound(0, 0)?, 1)?,
        RegionId::bound(0, 0)?
    );
    assert_eq!(
        fold.region(RegionId::bound(1, 0)?, 1)?,
        RegionId::bound(1, 1)?
    );
    let inner = reference(&mut store, RegionId::bound(0, 0)?)?;
    let outer = reference(&mut store, RegionId::bound(1, 0)?)?;
    let parameters = store.intern_list(&[inner, outer])?;
    let ty = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: inner,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let result = fold.apply(&mut store, ty)?;
    let mut ordinary = Bound::restore_regions(HashMap::from([
        (RegionId(1), RegionId(20)),
        (RegionId::bound(0, 0)?, RegionId::bound(0, 1)?),
    ]));
    assert_eq!(result, ordinary.apply(&mut store, ty)?);
    let foreign = reference(&mut store, RegionId(99))?;
    assert!(fold.apply(&mut store, foreign).is_err());
    Ok(())
}
#[test]
fn restore_failures_consume_work_and_warm_cache_keeps_its_budget() -> Result<(), &'static str> {
    let mut store = Store::default();
    let ty = reference(&mut store, RegionId(1))?;
    assert!(folder(0, 100).is_err());
    let mut fold = folder(1 << 20, 12)?;
    let result = fold.apply(&mut store, ty)?;
    let bytes = fold.region_retained_bytes()?;
    let mut failed = false;
    for _ in 0..16 {
        if let Ok(value) = fold.apply(&mut store, ty) {
            assert_eq!(value, result);
        } else {
            failed = true;
            break;
        }
    }
    assert!(failed);
    let steps = fold.steps;
    assert!(fold.apply(&mut store, ty).is_err());
    assert!(fold.steps > steps);
    assert_eq!(fold.region_retained_bytes()?, bytes);
    assert!(fold.work.is_empty());
    Ok(())
}
#[test]
fn signature_scratch_fails_before_allocation_and_is_restored_on_error() -> Result<(), &'static str>
{
    let mut store = Store::default();
    let unknown = reference(&mut store, RegionId(9))?;
    let parameters = store.intern_list(&vec![unknown; 128])?;
    let signature = store.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: byte(),
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let mut fold = folder(1 << 20, 10000)?;
    let bytes = fold.region_retained_bytes()?;
    fold.tighten_restore_bytes(bytes)?;
    assert!(fold.signature(&mut store, signature, 0).is_err());
    assert_eq!(fold.region_retained_bytes()?, bytes);
    let mut fold = folder(1 << 20, 10000)?;
    assert!(fold.signature(&mut store, signature, 0).is_err());
    assert_eq!(fold.region_owner().ok_or("owner lost")?.external, 0);
    let good = reference(&mut store, RegionId(1))?;
    assert!(fold.apply(&mut store, good).is_ok());
    fold.arguments.push(RegionId::STATIC);
    assert!(fold.apply(&mut store, good).is_err());
    Ok(())
}

// Frozen pre-budget representation, including its boxed opaque alternatives.
#[expect(
    dead_code,
    reason = "layout comparison with the exact previous private representation"
)]
enum PreviousRetained {
    Interval(std::ops::Range<u32>),
    Stream(Vec<std::ops::Range<u32>>),
}
#[expect(
    dead_code,
    reason = "layout comparison with the exact previous private representation"
)]
enum PreviousMode {
    Observe,
    Scopes(Vec<std::ops::Range<u32>>),
    Close(Box<()>),
    Export(Box<()>),
    Open {
        first: u32,
        next: u32,
    },
    Rebase {
        source: std::ops::Range<u32>,
        first: u32,
    },
    Restore {
        retained: PreviousRetained,
    },
}
#[expect(
    dead_code,
    reason = "layout comparison with the exact previous private representation"
)]
struct PreviousMapping {
    mode: PreviousMode,
    replacements: HashMap<RegionId, RegionId>,
}

#[test]
fn predicate_and_argument_scratch_keep_the_same_persistent_owner() -> Result<(), &'static str> {
    use crate::types::{
        ItemId,
        arguments::Argument,
        parameters::clauses::{Clause, Quantified},
        parameters::predicates::{Bound as Requirement, Predicate},
        traits::Application,
    };
    let mut store = Store::default();
    let args = store.intern_arguments(&[
        Argument::Lifetime(RegionId(1)),
        Argument::Lifetime(RegionId::bound(1, 0)?),
        Argument::Lifetime(RegionId::bound(0, 0)?),
    ])?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments: args,
    })?;
    let clause = Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty: byte(),
            bound: Requirement::Trait(application),
        },
    })?);
    let mut fold = folder(1 << 20, 10000)?;
    let result = fold.clause(&mut store, clause)?;
    let mut ordinary = Bound::restore_regions(HashMap::from([
        (RegionId(1), RegionId(20)),
        (RegionId::bound(0, 0)?, RegionId::bound(0, 1)?),
    ]));
    assert_eq!(result, ordinary.clause(&mut store, clause)?);
    let bytes = fold.region_retained_bytes()?;
    let steps = fold.steps;
    assert_eq!(fold.clause(&mut store, clause)?, result);
    assert!(fold.steps > steps);
    assert_eq!(fold.region_retained_bytes()?, bytes);
    assert_eq!(fold.region_owner().ok_or("owner lost")?.external, 0);
    let mut tiny = folder(1 << 20, 10000)?;
    let before = tiny.region_retained_bytes()?;
    tiny.tighten_restore_bytes(before)?;
    assert!(tiny.clause(&mut store, clause).is_err());
    assert!(tiny.applications.is_none());
    assert_eq!(tiny.region_retained_bytes()?, before);
    let mut tiny = folder(1 << 20, 10000)?;
    let before = tiny.region_retained_bytes()?;
    tiny.tighten_restore_bytes(before)?;
    assert!(tiny.apply_arguments(&mut store, args).is_err());
    assert_eq!(tiny.region_retained_bytes()?, before);
    Ok(())
}
