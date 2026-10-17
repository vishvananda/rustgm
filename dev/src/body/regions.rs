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
    TypeOutlives {
        ty: TypeId,
        shorter: RegionId,
    },
}
#[derive(Clone, Copy)]
pub struct Obligation {
    pub source: NodeKey,
    pub kind: Kind,
}
#[derive(Clone)]
pub struct Contract {
    pub source: NodeKey,
    pub signature: Option<TypeId>,
    pub universals: u32,
    pub assumptions: Vec<Pair>,
    pub parameters: Vec<crate::types::parameters::ParameterId>,
    pub type_assumptions: Vec<(TypeId, RegionId)>,
    pub obligations: Vec<Obligation>,
    pub proofs: Vec<(NodeKey, crate::types::lifetime_proof::Obligations)>,
}
impl Contract {
    /// Materialize demanded nominal requirements before borrowing immutable proof facts.
    /// # Errors
    /// Rejects malformed types and exhausted nominal preparation resources.
    pub fn prepare_outlives(
        &self,
        types: &mut crate::types::Store,
        bounds: &mut crate::types::outlives::Bounds,
    ) -> Result<(), &'static str> {
        if let Some(signature) = self.signature {
            bounds.prepare(types, signature)?;
        }
        for obligation in &self.obligations {
            if let Kind::WellFormed(ty) = obligation.kind {
                bounds.prepare(types, ty)?;
            }
        }
        Ok(())
    }
    fn constant(source: NodeKey) -> Self {
        Self {
            source,
            signature: None,
            universals: 0,
            assumptions: Vec::new(),
            parameters: Vec::new(),
            type_assumptions: Vec::new(),
            obligations: Vec::new(),
            proofs: Vec::new(),
        }
    }
}
pub struct Contracts<'a> {
    pub callables: &'a crate::types::callable::Templates,
    pub nominal_requirements: &'a crate::types::outlives::Bounds,
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
            index.insert(source, bodies.len());
            bodies.push(function_contract(facts, function, source)?);
        }
    }
    for initializer in &facts.initializers {
        index.insert(initializer.value, bodies.len());
        bodies.push(Contract::constant(initializer.value));
    }
    for (source, _) in facts.constant_bodies() {
        if index.insert(source, bodies.len()).is_some() {
            return Err("duplicate anonymous constant lifetime owner");
        }
        bodies.push(Contract::constant(source));
    }
    constructors(facts, &mut bodies, &mut index)?;
    let mut add = |body, source, kind| -> Result<(), &'static str> {
        let index = index
            .get(&body)
            .copied()
            .ok_or("lifetime obligation has no body owner")?;
        bodies[index].obligations.push(Obligation { source, kind });
        Ok(())
    };
    for (i, v) in facts.body_regions.iter().enumerate() {
        if matches!(v.origin, BodyRegionOrigin::QuantifiedProof) {
            continue;
        }
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
    for bound in &facts.type_outlives {
        add(
            bound.body,
            bound.source,
            Kind::TypeOutlives {
                ty: bound.ty,
                shorter: bound.shorter,
            },
        )?;
    }
    for function in &facts.functions {
        if let Some(body) = function.body {
            for predicate in facts
                .generics
                .predicates
                .get(&function.definition)
                .into_iter()
                .flatten()
            {
                // Associated equality formation follows its separate trait premise.
                // The result has its own well-formedness requirements; inspecting
                // the opaque projected subject cannot substitute for those checks.
                let ty = match predicate.value.bound {
                    crate::types::parameters::predicates::Bound::ProjectionEqual(result) => result,
                    _ => predicate.value.ty,
                };
                add(body, predicate.source, Kind::WellFormed(ty))?;
            }
        }
    }
    collect_proofs(facts, &mut bodies, &index)?;
    Ok(Contracts {
        bodies,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    })
}
fn collect_proofs(
    facts: &Declarations,
    bodies: &mut [Contract],
    index: &HashMap<NodeKey, usize>,
) -> Result<(), &'static str> {
    for proof in &facts.quantified_proofs {
        let index = index
            .get(&proof.body)
            .copied()
            .ok_or("quantified proof has no body owner")?;
        bodies[index]
            .proofs
            .push((proof.source, proof.proof.clone()));
    }
    Ok(())
}
fn constructors(
    facts: &Declarations,
    bodies: &mut Vec<Contract>,
    index: &mut HashMap<NodeKey, usize>,
) -> Result<(), &'static str> {
    for (ordinal, ty) in facts.definitions.iter().enumerate() {
        let Some(crate::types::Kind::FunctionItem {
            item, signature, ..
        }) = ty.and_then(|ty| facts.types.kind(ty))
        else {
            continue;
        };
        if item.0 as usize != ordinal {
            continue;
        }
        let Some(variant) = facts
            .members
            .constructors
            .get(&crate::resolve::model::DefId(item.0))
        else {
            continue;
        };
        let variant = facts.members.variants[variant.0 as usize];
        if facts
            .generics
            .parameters
            .contains_key(&crate::resolve::model::DefId(variant.owner.0))
        {
            continue;
        }
        let source = variant.node;
        if index.insert(source, bodies.len()).is_some() {
            return Err("duplicate constructor lifetime owner");
        }
        bodies.push(constructor(facts, variant, signature)?);
    }
    Ok(())
}
fn function_contract(
    facts: &Declarations,
    function: &crate::sema::model::Function,
    source: NodeKey,
) -> Result<Contract, &'static str> {
    let owner = function.definition;
    let scheme = facts.lifetimes.functions.get(&owner);
    let mut contract = parameter_contract(facts, owner, source, function.signature)?;
    contract.universals = scheme.map_or(0, |s| s.total);
    contract.assumptions = scheme.map_or_else(Vec::new, |s| s.bounds.clone());
    Ok(contract)
}
pub(super) fn constructor(
    facts: &Declarations,
    variant: crate::sema::members::Variant,
    signature: TypeId,
) -> Result<Contract, &'static str> {
    let Some(crate::types::Kind::Function { bound_regions, .. }) = facts.types.kind(signature)
    else {
        return Err("constructor has no lifetime signature");
    };
    let mut contract = parameter_contract(
        facts,
        crate::resolve::model::DefId(variant.owner.0),
        variant.node,
        signature,
    )?;
    contract.universals = bound_regions;
    // Signature well-formedness contributes the completed nominal requirements.
    Ok(contract)
}
fn parameter_contract(
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
    source: NodeKey,
    signature: TypeId,
) -> Result<Contract, &'static str> {
    let mut parameters = Vec::new();
    for parameter in facts.generics.parameters_in_scope(owner) {
        if let crate::types::arguments::Argument::Type(ty) = parameter.argument {
            let Some(crate::types::Kind::Parameter(id)) = facts.types.kind(ty) else {
                return Err("body parameter must have a rigid identity");
            };
            parameters.push(id);
        }
    }
    let type_assumptions = facts
        .generics
        .predicates_in_scope(owner)
        .filter_map(|p| {
            if let crate::types::parameters::predicates::Bound::Outlives(region) = p.value.bound {
                Some((p.value.ty, region))
            } else {
                None
            }
        })
        .collect();
    Ok(Contract {
        source,
        signature: Some(signature),
        universals: 0,
        assumptions: Vec::new(),
        parameters,
        type_assumptions,
        obligations: Vec::new(),
        proofs: Vec::new(),
    })
}
