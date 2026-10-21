//! Binder demand preserves local quantifiers and visits shared partial graphs once.
use super::*;
use crate::types::{Kind, TypeId, callable::Convention, infer::Domain};

fn pointer(context: &mut Context, child: TermId, binders: u32) -> Result<TermId, Error> {
    context.construct(
        Constructor::Function {
            bound_regions: binders,
            parameters: 1,
            unsafe_: false,
            convention: Convention::RUST,
        },
        &[child, child],
    )
}

#[test]
fn partial_pointer_distinguishes_outer_and_local_lifetimes() -> Result<(), Error> {
    let mut store = Store::default();
    let mut context = Context::default();
    let hole = context.variable(Domain::Any)?;
    for (depth, expected) in [(0, false), (1, true)] {
        let region = RegionId::bound(depth, 0).map_err(|_| Error::InvalidIdentity)?;
        let reference = context.construct(
            Constructor::Reference {
                mutable: false,
                region,
            },
            &[hole],
        )?;
        let signature = pointer(&mut context, reference, u32::from(depth == 0))?;
        let before = (
            store.len(),
            context.terms.len(),
            context.region_equalities.len(),
        );
        assert_eq!(
            context.contains_escaping_regions(&store, signature)?,
            expected
        );
        assert_eq!(
            (
                store.len(),
                context.terms.len(),
                context.region_equalities.len()
            ),
            before
        );
        let nested = pointer(&mut context, signature, 1)?;
        assert!(!context.contains_escaping_regions(&store, nested)?);
    }
    let region = RegionId::bound(1, 0).map_err(|_| Error::InvalidIdentity)?;
    let ty = store
        .intern(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })
        .map_err(|_| Error::InvalidIdentity)?;
    let known = context.known(&store, ty)?;
    let signature = pointer(&mut context, known, 0)?;
    assert!(context.contains_escaping_regions(&store, signature)?);
    Ok(())
}

#[test]
fn shared_unknown_graph_has_linear_demand_without_caching_future_solutions() -> Result<(), Error> {
    for depth in [8, 32, 128] {
        let store = Store::default();
        let mut context = Context::default();
        let hole = context.variable(Domain::Any)?;
        let mut graph = hole;
        for _ in 0..depth {
            graph = context.construct(Constructor::Tuple, &[graph, graph])?;
        }
        let before = context.steps;
        assert!(!context.contains_escaping_regions(&store, graph)?);
        assert!(context.steps - before <= 5 * depth + 1);
        let unit = context.known(&store, TypeId::UNIT)?;
        let region = RegionId::bound(0, 0).map_err(|_| Error::InvalidIdentity)?;
        let reference = context.construct(
            Constructor::Reference {
                mutable: false,
                region,
            },
            &[unit],
        )?;
        context.equate(&store, hole, reference)?;
        assert!(context.contains_escaping_regions(&store, graph)?);
    }
    Ok(())
}
