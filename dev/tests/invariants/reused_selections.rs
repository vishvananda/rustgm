//! Repeated canonical roots reuse completed proofs without erasing conditional evidence.
use super::tests;
use crate::{
    sema::{
        nominals::traits::State,
        traits::{assumptions::Assumptions, implementations::Availability, selection},
    },
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
    },
};

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn projected_nominal_roots_share_completed_suffix_proofs() -> Result {
    for count in [16, 32, 64, 128] {
        let (report, mut facts) = tests::declarations(
            "trait A{type Out:?Sized;}impl A for u8{type Out=u8;}
             struct W<T:A+?Sized>{tail:T::Out}
             impl<T:A+?Sized>A for W<T>{type Out=W<T>;}",
        )?;
        let item = ItemId(tests::definition(&report, "W")?.0);
        let ability = tests::definition(&report, "A")?;
        let mut ty = TypeId::primitive(Primitive::U8);
        let mut goals = Vec::new();
        for _ in 0..count {
            let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
            ty = facts.types.intern(Kind::Adt(item, arguments))?;
            goals.push(tests::predicate(
                &mut facts,
                ability,
                ty,
                GenericArgs::EMPTY,
            )?);
        }
        let mut state = State::instance(&mut facts, 0)?;
        for &goal in goals.iter().rev() {
            assert!(state.prove(&mut facts, goal, false)?);
        }
        let first = state.selection_steps();
        assert!(first < count * 512, "{count} roots required {first} work");
        for &goal in &goals {
            assert!(state.prove(&mut facts, goal, false)?);
        }
        let repeated = state.selection_steps() - first;
        assert!(
            repeated <= count * 4,
            "{count} roots repeated {repeated} work"
        );
        assert!(state.finish().is_empty());
    }
    Ok(())
}

#[test]
fn retained_builtin_requirements_are_not_replaced_with_empty_proofs() -> Result {
    let (_, mut facts) = tests::declarations("struct S{field:u8}fn f(){}")?;
    let goal = Predicate {
        ty: TypeId::primitive(Primitive::U8),
        bound: Bound::Sized,
    };
    let assumptions = Assumptions::default();
    let mut reuse = selection::Reuse::default();
    for first in [100, 200] {
        let (mut context, inferred) = tests::instantiate(&mut facts, goal)?;
        let mut caller = tests::caller(&assumptions, None);
        caller.first_region = first;
        caller.retain_sized = true;
        let selection::Selection::Proven(evidence) = selection::select_reusing(
            &mut facts,
            &mut context,
            &inferred,
            caller,
            (Availability::Validated, false),
            &mut reuse,
        )?
        else {
            return Err("missing retained Sized proof".into());
        };
        assert_eq!(evidence.sized, [goal.ty]);
        assert_eq!(evidence.proof.regions, first..first);
    }
    Ok(())
}
