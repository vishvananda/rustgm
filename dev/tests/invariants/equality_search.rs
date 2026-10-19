//! Implementation conditions keep quantified equality witnesses inside their proof scope.
use super::{quantified_tests::valid, tests};
use crate::{
    sema::nominals::traits::State,
    types::{
        RegionId,
        arguments::Argument,
        parameters::clauses::{Clause, Quantified},
    },
};
use std::sync::Arc;

#[test]
fn equality_normalization_under_a_universal_impl_condition_keeps_private_witnesses()
-> Result<(), Box<dyn std::error::Error>> {
    for required in ["'a", "'static"] {
        let source = format!(
            "trait A<'a>{{type Out;}}trait B<'a>{{}}
             impl<'a,X:A<'a,Out=&{required} u8>>B<'a>for X{{}}
             fn caller<T>()where for<'a>T:A<'a,Out=&'a u8>{{}}"
        );
        let (report, mut facts) = tests::declarations(&source)?;
        let owner = tests::definition(&report, "caller")?;
        let environment = facts.generics.parameter_environment(&facts.types, owner)?;
        let Argument::Type(subject) = facts.generics.parameters[&owner][0].argument else {
            return Err("missing caller parameter".into());
        };
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let predicate = tests::predicate(
            &mut facts,
            tests::definition(&report, "B")?,
            subject,
            arguments,
        )?;
        let goal = Clause::ForAll(facts.types.intern_quantified(Quantified {
            bound_regions: 1,
            predicate,
        })?);
        let mut state = State::new(&mut facts, Some(Arc::new(environment)), Some((0, 100)))?;
        assert!(state.prove_clause(&mut facts, goal, false)?);
        let proof = state.finish();
        assert!(proof.scope_count() > 0);
        assert!(proof.regions.end > 102);
        assert_eq!(valid(&facts, &proof, 0)?, required == "'a");
        let mut foreign = State::new(&mut facts, None, Some((0, 100)))?;
        assert!(!foreign.prove_clause(&mut facts, goal, false)?);
    }
    Ok(())
}
