//! Matched recursive inputs share recipes and bindings instead of copying prefixes.
use super::{State, View};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    copy::Properties,
    outlives::Projection,
    parameters::{
        clauses::Quantified,
        predicates::{Bound, Predicate},
    },
    projection::{Application, Schema},
    region_graph::Region,
    relations::{Constraints, Scope},
    traits,
    variance::Variances,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn reference(types: &mut Store, region: RegionId, referent: TypeId) -> Result<TypeId> {
    Ok(types.intern(Kind::Reference {
        region,
        referent,
        mutable: false,
    })?)
}
fn output(types: &mut Store, subject: TypeId) -> Result<TypeId> {
    types.declare_projection(
        ItemId(3),
        Schema {
            trait_item: ItemId(2),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let trait_application = types.intern_trait_application(traits::Application {
        item: ItemId(2),
        arguments: GenericArgs::EMPTY,
    })?;
    Ok(types.intern(Kind::Projection(Application {
        item: ItemId(3),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?)
}
fn pairs(types: &mut Store, roots: &[(TypeId, TypeId)], count: u32) -> Result<State> {
    let mut state = State::default();
    for &(pattern, _) in roots {
        let id = types.intern_quantified(Quantified {
            bound_regions: count,
            predicate: Predicate {
                ty: pattern,
                bound: Bound::Outlives(RegionId::STATIC),
            },
        })?;
        state.assume(types, id)?;
    }
    Ok(state)
}
fn verify(state: &mut State, proof: &mut Constraints<'_>, ty: TypeId) -> Result {
    let view = View {
        projection: Projection { ty, depth: 0 },
        scope: Scope(0),
    };
    let alternatives = state.alternatives(proof, view, Region::STATIC)?;
    assert_eq!(alternatives.len(), 1);
    let result = proof.graph.verify_any(&alternatives)?;
    proof.graph.require_verification(result, 0)?;
    Ok(())
}

#[test]
fn growing_matched_tails_share_slot_bindings_and_conditional_guard_sets() -> Result {
    for width in [16_u16, 64, 256, 1024] {
        for captured in [false, true] {
            let mut types = Store::default();
            let mut pattern = TypeId::UNIT;
            let mut actual = TypeId::UNIT;
            let mut roots = Vec::new();
            for slot in 0..width {
                pattern = reference(
                    &mut types,
                    RegionId::bound(u16::from(captured), slot)?,
                    pattern,
                )?;
                actual = reference(&mut types, RegionId::bound(0, slot + width)?, actual)?;
                roots.push((output(&mut types, pattern)?, output(&mut types, actual)?));
            }
            let mut state = pairs(&mut types, &roots, u32::from(width))?;
            let variances = Variances::default();
            let mut proof = Constraints::new(&types, &variances, 2 * u32::from(width), 1_000_000)?;
            if captured {
                for slot in 0..width {
                    let a = RegionId::bound(0, slot)?;
                    let b = RegionId::bound(0, slot + width)?;
                    proof.assume(a, b)?;
                    proof.assume(b, a)?;
                }
            }
            for &(_, actual) in &roots {
                verify(&mut state, &mut proof, actual)?;
            }
            let work = state.steps();
            assert!(
                work < 180 * usize::from(width),
                "width={width}, captures={captured}, work={work}"
            );
            for &(_, actual) in &roots {
                verify(&mut state, &mut proof, actual)?;
            }
            assert!(state.steps() - work < 3 * usize::from(width));
            eprintln!("matched tails width={width} captures={captured} steps={work}");
            proof.solve()?;
        }
    }
    Ok(())
}

#[test]
fn shared_wide_argument_lists_and_rigid_candidate_index_are_additive() -> Result {
    for width in [16_u16, 64, 256, 1024] {
        let mut types = Store::default();
        let inputs = (0..width)
            .map(|slot| Ok(Argument::Lifetime(RegionId::bound(0, slot)?)))
            .collect::<std::result::Result<Vec<_>, &'static str>>()?;
        let arguments = types.intern_arguments(&inputs)?;
        let mut roots = Vec::new();
        for item in 0..width {
            let subject = types.intern(Kind::Adt(ItemId(100 + u32::from(item)), arguments))?;
            let projected = output(&mut types, subject)?;
            roots.push((projected, projected));
        }
        let mut state = pairs(&mut types, &roots, u32::from(width))?;
        let variances = Variances::default();
        let mut proof = Constraints::new(&types, &variances, u32::from(width), 1_000_000)?;
        for &(_, actual) in &roots {
            verify(&mut state, &mut proof, actual)?;
        }
        let work = state.steps();
        assert!(
            work < 180 * usize::from(width),
            "width={width}, work={work}"
        );
        // Exactly one candidate per distinct rigid shape: this catches both an
        // all-environment scan and a repeated traversal of the shared wide list.
        assert_eq!(proof.steps, usize::from(width));
        eprintln!("wide shared arguments width={width} steps={work}");
        proof.solve()?;
    }
    Ok(())
}
