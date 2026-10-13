//! Group typed lifetime facts by body without repeating resolution or inference.
use super::ir::NodeKey;
use crate::{
    sema::model::{AdjustmentKind, BodyRegionOrigin, Declarations},
    types::{RegionId, TypeId, outlives::Pair},
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub enum Kind {
    Variable {
        id: RegionId,
        needs_provenance: bool,
    },
    /// Region upper bound from function-owned storage; loan liveness is separate.
    LocalBorrow(RegionId),
    Equality(Pair),
    Outlives(Pair),
    Coercion {
        actual: TypeId,
        expected: TypeId,
        unsize: bool,
        reference: Option<crate::sema::model::ReferenceConversion>,
    },
    WellFormed(TypeId),
}
#[derive(Clone, Copy)]
pub struct Obligation {
    pub source: NodeKey,
    pub kind: Kind,
}
pub struct Contract {
    pub source: NodeKey,
    pub signature: Option<TypeId>,
    pub universals: u32,
    pub assumptions: Vec<Pair>,
    pub obligations: Vec<Obligation>,
}
pub struct Contracts<'a> {
    pub callables: &'a crate::types::callable::Templates,
    pub nominal_bounds: &'a [(crate::types::ItemId, Pair)],
    pub bodies: Vec<Contract>,
}
/// All variables precede obligations that use them. Indices remain the globally
/// allocated semantic region identities, even when body declarations interleave.
///
/// # Errors
/// Rejects missing body owners and overflowing inference identities.
pub fn collect(facts: &Declarations) -> Result<Contracts<'_>, &'static str> {
    let mut bodies = Vec::new();
    let mut index = HashMap::new();
    for function in &facts.functions {
        if let Some(source) = function.body {
            let scheme = facts.lifetimes.functions.get(&function.definition);
            index.insert(source, bodies.len());
            bodies.push(Contract {
                source,
                signature: Some(function.signature),
                universals: scheme.map_or(0, |s| s.total),
                assumptions: scheme.map_or_else(Vec::new, |s| s.bounds.clone()),
                obligations: Vec::new(),
            });
        }
    }
    for initializer in &facts.initializers {
        index.insert(initializer.value, bodies.len());
        bodies.push(Contract {
            source: initializer.value,
            signature: None,
            universals: 0,
            assumptions: Vec::new(),
            obligations: Vec::new(),
        });
    }
    let mut add = |body, source, kind| -> Result<(), &'static str> {
        let index = index
            .get(&body)
            .copied()
            .ok_or("lifetime obligation has no body owner")?;
        bodies[index].obligations.push(Obligation { source, kind });
        Ok(())
    };
    for (i, v) in facts.body_regions.iter().enumerate() {
        add(
            v.body,
            v.source,
            Kind::Variable {
                id: RegionId(u32::try_from(i + 1).map_err(|_| "too many lifetime variables")?),
                needs_provenance: matches!(
                    v.origin,
                    BodyRegionOrigin::ExpressionBorrow | BodyRegionOrigin::PatternBorrow
                ),
            },
        )?;
    }
    for e in &facts.region_equalities {
        add(e.body, e.body, Kind::Equality((e.left, e.right)))?;
    }
    for b in &facts.outlives {
        add(b.body, b.source, Kind::Outlives((b.longer, b.shorter)))?;
    }
    for c in &facts.coercions {
        add(
            c.body,
            c.source,
            Kind::Coercion {
                actual: c.actual,
                expected: c.expected,
                unsize: c.kind == AdjustmentKind::ArrayToSlice,
                reference: c.reference,
            },
        )?;
    }
    for w in &facts.well_formed {
        add(w.body, w.source, Kind::WellFormed(w.ty))?;
    }
    Ok(Contracts {
        bodies,
        callables: &facts.lifetimes.callables,
        nominal_bounds: &facts.lifetimes.nominal_bounds,
    })
}
