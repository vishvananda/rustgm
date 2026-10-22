//! Declaration obligations reach fresh scoped lifetime proofs without becoming assumptions.
use super::Pending;
use crate::{
    resolve::unit::{FileId, NodeKey},
    sema::traits::{Requirements, formation::Formation},
    syntax::ast::NodeId,
    types::{
        ItemId, Kind, RegionId, Store,
        callable::Templates,
        outlives::{Bounds, Queries},
        parameters::{
            Parameter, ParameterKind,
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
        relations::Constraints,
        variance::Variances,
    },
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn inherited_universal_outlives_is_required_instead_of_assumed() -> Result {
    let mut types = Store::default();
    let parameter = types.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = types.intern(Kind::Parameter(parameter))?;
    let predicate = Predicate {
        ty,
        bound: Bound::Outlives(RegionId::bound(0, 0)?),
    };
    let clause = Clause::ForAll(types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate,
    })?);
    let formation = Formation::Ordinary(
        predicate,
        Arc::new(Requirements {
            implied: Vec::new(),
            results: Vec::new(),
            required: vec![clause],
            types: Vec::new(),
            regions: Vec::new(),
        }),
    );
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    let source = NodeKey {
        file: FileId(0),
        node: NodeId::new(0),
    };
    for assumption in 0..3 {
        let mut queries = Queries::new(&types, &bounds, &templates);
        let pending = Pending::collect(&types, source, &formation, &mut queries, &mut 0)?
            .ok_or("universal outlives obligation was discarded")?;
        let mut proof = Constraints::new(&types, &variances, 1, 10_000)?;
        proof.parameter(parameter)?;
        match assumption {
            1 => proof.assume_type_outlives(&mut queries, ty, RegionId::bound(0, 0)?)?,
            2 => proof.assume_outlives_clause(&mut queries, clause)?,
            _ => {}
        }
        pending.require(&mut proof, &mut queries)?;
        assert_eq!(proof.solve().is_ok(), assumption == 2);
    }
    Ok(())
}

#[test]
fn other_quantified_bound_kinds_do_not_become_lifetime_obligations() -> Result {
    let mut types = Store::default();
    let predicate = Predicate {
        ty: crate::types::TypeId::UNIT,
        bound: Bound::Sized,
    };
    let clause = Clause::ForAll(types.intern_quantified(Quantified {
        bound_regions: 1,
        predicate,
    })?);
    let formation = Formation::Ordinary(
        predicate,
        Arc::new(Requirements {
            implied: Vec::new(),
            results: Vec::new(),
            required: vec![clause],
            types: Vec::new(),
            regions: Vec::new(),
        }),
    );
    let bounds = Bounds::default();
    let templates = Templates::default();
    let mut queries = Queries::new(&types, &bounds, &templates);
    let source = NodeKey {
        file: FileId(0),
        node: NodeId::new(0),
    };
    assert!(Pending::collect(&types, source, &formation, &mut queries, &mut 0)?.is_none());
    Ok(())
}
