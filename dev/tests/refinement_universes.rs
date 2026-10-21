//! Exported input types need older caller witnesses and retained scoped equations.
use rustgm::types::{
    Kind, Primitive, RegionId, Store, TypeId,
    callable::Templates,
    infer::{Constructor, Context, Domain, arguments::Argument, refinement::Refinement},
    lifetime_proof::{Obligations, ScopeId},
    outlives::{Bounds, Queries},
    relations::Constraints,
    substitute::{Bound, ExportSource},
    variance::Variances,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug)]
enum Debt {
    Independent,
    Longer,
    Shorter,
    IndirectEqual,
}

fn export(store: &mut Store) -> TestResult<Obligations> {
    let mut caller = Context::default();
    let input = caller.variable(Domain::Any)?;
    let (mut query, copied) = Refinement::new(store, &mut caller, &[Argument::Type(input)])?;
    let Argument::Type(copied) = copied[0] else {
        return Err("missing imported input".into());
    };
    let byte = query
        .context
        .known(store, TypeId::primitive(Primitive::U8))?;
    let reference = query.context.construct(
        Constructor::Reference {
            mutable: false,
            region: RegionId(2),
        },
        &[byte],
    )?;
    query.context.equate(store, copied, reference)?;
    let mut closing = Bound::export_regions(&[ExportSource::Private(2..4)], 4)?;
    let committed = query.commit(store, &mut caller, &mut closing)?;
    assert!(committed.changed);
    assert!(committed.equalities.is_empty());
    assert!(caller.region_equalities.is_empty());
    assert_eq!(
        caller.head(store, input)?,
        Some(Constructor::Reference {
            mutable: false,
            region: RegionId(4),
        })
    );
    assert_eq!(closing.next_region(), Some(5));
    assert_eq!(
        closing.exported_regions().collect::<Vec<_>>(),
        [(RegionId(2), RegionId(4))]
    );
    Ok(Obligations::new(2))
}

fn prove(debt: Debt, captured: bool, permission: bool, bridge: bool) -> TestResult<bool> {
    let mut store = Store::default();
    let mut proof = export(&mut store)?;
    let scope = proof.scope(ScopeId::ROOT, 1)?;
    let universal = RegionId::bound(0, 0)?;
    let mut private = Obligations::new(2);
    private.regions.end = 4;
    if bridge {
        private.equalities.push((RegionId(2), RegionId(4)));
    }
    match debt {
        Debt::Independent => {}
        Debt::Longer => private.outlives.push((RegionId(2), universal)),
        Debt::Shorter => private.outlives.push((universal, RegionId(2))),
        Debt::IndirectEqual => {
            private
                .equalities
                .extend([(RegionId(2), RegionId(3)), (RegionId(3), universal)]);
        }
    }
    proof.append_at(private, scope)?;
    // Numeric allocation follows the private query, but logical ownership is
    // still the older root: this tail is outside the scoped private block.
    proof.regions.end = 5;
    if captured {
        proof.equalities.push((RegionId(4), universal));
    }
    let variances = Variances::default();
    let bounds = Bounds::default();
    let callables = Templates::default();
    let mut constraints = Constraints::new(&store, &variances, u32::from(captured), 100_000)?;
    if permission {
        constraints.assume(universal, RegionId::STATIC)?;
    }
    let mut queries = Queries::new(&store, &bounds, &callables);
    proof.require(&mut constraints, &mut queries)?;
    Ok(constraints.prove().is_ok())
}

#[test]
fn older_witness_proofs_match_the_independently_qualified_source_hypotheses() -> TestResult {
    for (debt, captured, permission, accepted) in [
        (Debt::Independent, false, false, true),
        (Debt::Longer, false, false, true),
        (Debt::Shorter, false, false, false),
        (Debt::Longer, true, false, false),
        (Debt::Longer, true, true, true),
        (Debt::IndirectEqual, false, false, false),
    ] {
        assert_eq!(
            prove(debt, captured, permission, true)?,
            accepted,
            "{debt:?}, captured={captured}, permission={permission}"
        );
    }
    Ok(())
}

#[test]
fn disconnected_query_witness_does_not_establish_the_exported_type() -> TestResult {
    for debt in [Debt::Shorter, Debt::IndirectEqual] {
        assert!(prove(debt, false, false, false)?);
        assert!(!prove(debt, false, false, true)?);
    }
    Ok(())
}

fn reference(store: &mut Store, region: RegionId) -> TestResult<TypeId> {
    Ok(store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::primitive(Primitive::U8),
    })?)
}

#[test]
fn only_encountered_private_lifetimes_allocate_and_callable_locals_remain_local() -> TestResult {
    let mut store = Store::default();
    let caller = reference(&mut store, RegionId(1))?;
    let private = reference(&mut store, RegionId(2))?;
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let parameters = store.intern_list(&[caller, private, private, local])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: private,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let mut closing = Bound::export_regions(
        &[ExportSource::Caller(1..2), ExportSource::Private(2..4)],
        5,
    )?;
    let result = closing.apply(&mut store, function)?;
    let exported = reference(&mut store, RegionId(5))?;
    let expected = store.intern_list(&[caller, exported, exported, local])?;
    assert_eq!(
        store.kind(result),
        Some(Kind::Function {
            bound_regions: 1,
            parameters: expected,
            result: exported,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })
    );
    assert_eq!(closing.next_region(), Some(6));
    assert_eq!(
        closing.exported_regions().collect::<Vec<_>>(),
        [(RegionId(2), RegionId(5))]
    );
    assert_eq!(closing.apply(&mut store, function)?, result);
    assert_eq!(closing.apply(&mut store, exported)?, exported);
    assert_eq!(closing.next_region(), Some(6));
    Ok(())
}

#[test]
fn resumption_preserves_old_exports_and_excludes_aliases_and_allocation_gaps() -> TestResult {
    let mut store = Store::default();
    let mut closing = Bound::export_regions(
        &[ExportSource::Caller(1..2), ExportSource::Private(2..4)],
        5,
    )?;
    let original = reference(&mut store, RegionId(2))?;
    let first = closing.apply(&mut store, original)?;
    closing.resume_export(
        &[ExportSource::Private(6..7), ExportSource::Caller(7..9)],
        10,
    )?;
    let later = reference(&mut store, RegionId(6))?;
    let actual = closing.apply(&mut store, later)?;
    assert_eq!(actual, reference(&mut store, RegionId(10))?);
    assert_eq!(closing.apply(&mut store, original)?, first);
    for id in [1, 7, 8] {
        let ty = reference(&mut store, RegionId(id))?;
        assert_eq!(closing.apply(&mut store, ty)?, ty);
    }
    assert!(
        closing
            .resume_export(&[ExportSource::Caller(4..5)], 11)
            .is_err()
    );
    assert!(closing.resume_export(&[], 10).is_err());
    assert_eq!(closing.next_region(), Some(11));
    assert_eq!(
        closing.exported_regions().collect::<Vec<_>>(),
        [(RegionId(2), RegionId(5)), (RegionId(6), RegionId(10)),]
    );
    Ok(())
}

#[test]
fn unowned_and_universal_lifetimes_reject_before_caller_mutation() -> TestResult {
    for region in [
        RegionId(4),
        RegionId(9),
        RegionId(11),
        RegionId::bound(0, 0)?,
    ] {
        // A failed fold is discarded; each rejection has its own fresh query.
        let mut store = Store::default();
        let mut caller = Context::default();
        let input = caller.variable(Domain::Any)?;
        let (mut query, copied) = Refinement::new(&store, &mut caller, &[Argument::Type(input)])?;
        let Argument::Type(copied) = copied[0] else {
            return Err("missing imported input".into());
        };
        let ty = reference(&mut store, region)?;
        let term = query.context.known(&store, ty)?;
        query.context.equate(&store, copied, term)?;
        let mut closing = Bound::export_regions(
            &[ExportSource::Caller(1..2), ExportSource::Private(2..4)],
            5,
        )?;
        closing.resume_export(
            &[ExportSource::Private(6..7), ExportSource::Caller(7..9)],
            10,
        )?;
        let sentinel = caller.variable(Domain::Any)?;
        assert!(query.commit(&mut store, &mut caller, &mut closing).is_err());
        assert_eq!(caller.unresolved_domain(input)?, Some(Domain::Any));
        assert_eq!(caller.variable(Domain::Any)?.0, sentinel.0 + 1);
        assert!(caller.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn shared_canonical_prefixes_are_exported_once_without_allocating_unused_witnesses() -> TestResult {
    for count in [16, 128, 1024, 4096] {
        let mut store = Store::default();
        let mut ty = reference(&mut store, RegionId(2))?;
        let mut roots = Vec::new();
        for _ in 0..count {
            let list = store.intern_list(&[ty])?;
            ty = store.intern(Kind::Tuple(list))?;
            roots.push(ty);
        }
        let mut closing = Bound::export_regions(&[ExportSource::Private(2..8192)], 8192)?;
        let mut expected = reference(&mut store, RegionId(8192))?;
        for original in roots {
            let list = store.intern_list(&[expected])?;
            expected = store.intern(Kind::Tuple(list))?;
            assert_eq!(closing.apply(&mut store, original)?, expected);
        }
        eprintln!(
            "export prefixes={count} steps={} witnesses={}",
            closing.steps,
            closing.exported_regions().len()
        );
        assert!(closing.steps <= 12 * count + 32);
        assert_eq!(closing.exported_regions().len(), 1);
        assert_eq!(closing.next_region(), Some(8193));
    }
    Ok(())
}
