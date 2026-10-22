//! A field's declared marker does not discharge its parent application's lifetime debt.
use super::{Decision, Frame, Predicate, Search, size_evidence_tests::valid, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{
        ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument, auto_traits::AutoTrait,
        parameters::predicates::Bound,
    },
};
use std::sync::Arc;

#[test]
fn projected_auto_promises_preserve_parent_proofs_in_each_scope()
-> Result<(), Box<dyn std::error::Error>> {
    for (bound, expected) in [("", true), (":'static", false)] {
        let source = format!(
            "trait A{{type Out:Send;}}impl<'a{bound}>A for &'a u8{{type Out=u8;}}
             struct S<T:A>{{tail:T::Out}}fn caller<T:A>(){{}}"
        );
        let (report, mut facts) = tests::declarations(&source)?;
        let owner = tests::definition(&report, "caller")?;
        // Even an available conditional result summary cannot replace the
        // parent's proof. Exercise the selector independently of publication order.
        let member = ItemId(tests::definition(&report, "Out")?.0);
        let promises = Arc::new(crate::support::hash::HashMap::from_iter([(
            member,
            crate::types::projection::Builtins::bound(Bound::Auto(AutoTrait::Send)),
        )]));
        let environment = Arc::new(
            facts
                .generics
                .parameter_environment(&facts.types, owner)?
                .with_projection_promises(Some(promises)),
        );
        let referent = facts.types.intern(Kind::Reference {
            mutable: false,
            region: RegionId(10),
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let arguments = facts.types.intern_arguments(&[Argument::Type(referent)])?;
        let wrapper = facts.types.intern(Kind::Adt(
            ItemId(tests::definition(&report, "S")?.0),
            arguments,
        ))?;
        let predicate = Predicate {
            ty: wrapper,
            bound: Bound::Auto(AutoTrait::Send),
        };
        for first in [100, 200] {
            let assumptions = Assumptions::default();
            let (mut context, goal) = tests::instantiate(&mut facts, predicate)?;
            let mut caller = tests::caller(&assumptions, Some(&environment));
            caller.first_region = first;
            let mut search = Search::new(caller, Availability::Validated);
            let frame = Frame::new(&facts, &mut context, &goal, first)?;
            let Decision::Proven(root) = search.run(&mut facts, frame)? else {
                return Err("missing conditional projected auto-trait proof".into());
            };
            let evidence = search.evidence(root)?;
            assert_eq!(evidence.proof.regions.start, first);
            assert!(
                !evidence.proof.is_empty(),
                "projected marker discarded its parent proof"
            );
            assert_eq!(valid(&facts, &evidence.proof)?, expected);
        }
    }
    Ok(())
}
