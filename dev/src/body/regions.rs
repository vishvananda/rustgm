//! Group typed lifetime facts by body without repeating resolution or inference.
pub use crate::sema::nested_outputs::Link as OutputLink;
mod allocation;
mod anonymous;
mod clause_terms;
mod incremental;
mod proof;
use super::ir::{BodyOrigin, NodeKey, ProofSite};
pub use crate::sema::model::BuiltinRequirements;
use crate::support::hash::HashMap;
use crate::{
    sema::model::Declarations,
    types::{RegionId, TypeId, outlives::Pair},
};
use anonymous::header as anonymous_header;
pub use incremental::Collector;
pub use proof::Proof;

#[derive(Clone, Copy)]
pub enum Kind {
    Variable {
        id: RegionId,
        needs_provenance: bool,
    },
    Membership {
        region: RegionId,
        captures: crate::types::GenericArgs,
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
    /// Declaration formation under one preserved clause binder, not a universal use.
    ScopedWellFormed {
        ty: TypeId,
        bound_regions: u32,
    },
    TypeOutlives {
        ty: TypeId,
        shorter: RegionId,
    },
}
#[derive(Clone, Copy)]
pub struct Obligation {
    pub source: ProofSite,
    pub kind: Kind,
}
#[derive(Clone)]
pub struct Contract {
    pub key: super::ir::BodyKey,
    pub source: BodyOrigin,
    pub signature: Option<TypeId>,
    pub universals: u32,
    pub assumptions: Vec<Pair>,
    pub parameters: Vec<crate::types::parameters::ParameterId>,
    pub type_assumptions: Vec<crate::types::parameters::clauses::Clause>,
    pub obligations: Vec<Obligation>,
    pub proofs: Vec<Proof>,
    pub builtins: Option<std::sync::Arc<crate::sema::model::BuiltinRequirements>>,
    pub output: Option<std::sync::Arc<crate::sema::nested_outputs::Link>>,
    /// Specialized nested inputs can be discharged without granting output formation.
    pub require_result_formation: bool,
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
            if let Kind::WellFormed(ty) | Kind::ScopedWellFormed { ty, .. } = obligation.kind {
                bounds.prepare(types, ty)?;
            }
        }
        Ok(())
    }
    fn constant(source: NodeKey) -> Self {
        Self {
            key: super::ir::BodyKey::Anonymous(source),
            source: source.into(),
            signature: None,
            universals: 0,
            assumptions: Vec::new(),
            parameters: Vec::new(),
            type_assumptions: Vec::new(),
            obligations: Vec::new(),
            proofs: Vec::new(),
            builtins: None,
            output: None,
            require_result_formation: false,
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
    let mut index = HashMap::default();
    for function in &facts.functions {
        if let Some(source) = function.body {
            index.insert(source, bodies.len());
            bodies.push(function_contract(facts, function, source)?);
        }
    }
    for closure in facts.closures.all() {
        if index
            .insert(closure.semantic_source(), bodies.len())
            .is_some()
        {
            return Err("duplicate closure lifetime owner");
        }
        bodies.push(closure_contract(facts, closure)?);
    }
    for initializer in &facts.initializers {
        index.insert(initializer.semantic_source(), bodies.len());
        bodies.push(initializer_contract(facts, initializer)?);
    }
    for (source, _) in facts.constant_bodies() {
        if index.insert(source, bodies.len()).is_some() {
            return Err("duplicate anonymous constant lifetime owner");
        }
        let mut contract = Contract::constant(source);
        anonymous_header(facts, &mut contract)?;
        bodies.push(contract);
    }
    constructors(facts, &mut bodies, &mut index)?;
    for contract in &mut bodies {
        output_interface(facts, contract)?;
    }
    incremental::Cursor::default().visit(facts, |body, source, fact| {
        let target = index
            .get(&body)
            .copied()
            .ok_or("lifetime obligation has no body owner")?;
        match fact {
            incremental::Fact::Obligation(_, kind) => {
                bodies[target].obligations.push(Obligation {
                    source: source.into(),
                    kind,
                });
            }
            incremental::Fact::Proof(proof) => {
                bodies[target].proofs.push(Proof::from_source(proof));
            }
            // One pass after analysis sees final origins: a reserved region
            // still belongs to proof evidence.
            incremental::Fact::Reserved(_) => {}
        }
        Ok(())
    })?;
    let mut add = |body, source: NodeKey, kind| -> Result<(), &'static str> {
        let target = index
            .get(&body)
            .copied()
            .ok_or("lifetime obligation has no body owner")?;
        bodies[target].obligations.push(Obligation {
            source: source.into(),
            kind,
        });
        Ok(())
    };
    for function in &facts.functions {
        if let Some(body) = function.body {
            clause_terms::collect(facts, function.definition, |source, kind| {
                add(body, source, kind)
            })?;
            opaque_members(facts, function.definition, |source, kind| {
                add(body, source, kind)
            })?;
        }
    }
    for (source, requirements) in &facts.generics.builtin_requirements {
        let index = index
            .get(source)
            .copied()
            .ok_or("builtin requirement has no body contract")?;
        bodies[index].builtins = Some(std::sync::Arc::new(requirements.clone()));
    }
    super::length_methods::append_contract(facts, &mut bodies)?;
    Ok(Contracts {
        bodies,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    })
}
fn opaque_members(
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
    mut add: impl FnMut(NodeKey, Kind) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for id in facts.opaques.owners.get(&owner).into_iter().flatten() {
        let declaration = facts
            .opaques
            .declarations
            .get(id)
            .ok_or("missing opaque result declaration")?;
        let Some(crate::types::Kind::Opaque(_, captures)) =
            facts.types.kind(declaration.application)
        else {
            return Err("invalid opaque capture application");
        };
        for &region in &declaration.free_regions {
            add(declaration.source, Kind::Membership { region, captures })?;
        }
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
            .constructor(crate::resolve::model::DefId(item.0))
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
        let source = variant.origin.require_diagnostic()?;
        if facts.members.at_node(source).is_some() && index.insert(source, bodies.len()).is_some() {
            return Err("duplicate constructor lifetime owner");
        }
        bodies.push(constructor(
            facts,
            crate::resolve::model::DefId(item.0),
            variant,
            signature,
        )?);
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
    let mut contract = parameter_contract(facts, owner, source, Some(function.signature))?;
    contract.universals = scheme.map_or(0, |s| s.total);
    contract.assumptions = scheme.map_or_else(Vec::new, |s| s.bounds.clone());
    Ok(contract)
}
fn closure_contract(
    facts: &Declarations,
    closure: &crate::sema::closures::Body,
) -> Result<Contract, &'static str> {
    let signature = closure
        .execution
        .map_or(closure.signature, |execution| execution.signature);
    let mut contract = if let Some(owner) = closure.owner {
        parameter_contract(facts, owner, closure.semantic_source(), Some(signature))?
    } else {
        Contract::constant(closure.semantic_source())
    };
    contract.key = super::ir::BodyKey::Closure(closure.identity);
    contract.signature = Some(signature);
    contract.universals = closure
        .execution
        .map_or(closure.universals, |execution| execution.universals);
    if let Some(scheme) = closure
        .owner
        .and_then(|owner| facts.lifetimes.functions.get(&owner))
    {
        contract.assumptions.clone_from(&scheme.bounds);
    }
    Ok(contract)
}
fn initializer_contract(
    facts: &Declarations,
    initializer: &crate::sema::model::Initializer,
) -> Result<Contract, &'static str> {
    let Some(schema) = facts.associated.schema(initializer.definition) else {
        let mut contract = Contract::constant(initializer.semantic_source());
        contract.key = super::ir::BodyKey::Declaration(initializer.definition);
        return Ok(contract);
    };
    let mut contract = parameter_contract(
        facts,
        initializer.definition,
        initializer.semantic_source(),
        None,
    )?;
    contract.universals = schema.captures;
    contract.assumptions.clone_from(&schema.bounds);
    Ok(contract)
}
pub(super) fn constructor(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
    variant: crate::sema::members::Variant,
    signature: TypeId,
) -> Result<Contract, &'static str> {
    callable(
        facts,
        crate::resolve::model::DefId(variant.owner.0),
        definition,
        variant.origin.require_diagnostic()?,
        signature,
    )
}
pub(crate) fn callable(
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
    definition: crate::resolve::model::DefId,
    source: NodeKey,
    signature: TypeId,
) -> Result<Contract, &'static str> {
    let Some(crate::types::Kind::Function { bound_regions, .. }) = facts.types.kind(signature)
    else {
        return Err("constructor has no lifetime signature");
    };
    let mut contract = parameter_contract(facts, owner, source, Some(signature))?;
    contract.key = super::ir::BodyKey::Declaration(definition);
    contract.universals = bound_regions;
    // Signature well-formedness contributes the completed nominal requirements.
    Ok(contract)
}
fn parameter_contract(
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
    source: NodeKey,
    signature: Option<TypeId>,
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
    let mut type_assumptions = Vec::new();
    for predicate in facts.generics.predicates_in_scope(owner) {
        if let Some(clause) = predicate.value.caller_outlives(&facts.types)? {
            type_assumptions.push(clause);
        }
    }
    Ok(Contract {
        key: super::ir::BodyKey::Declaration(owner),
        source: source.into(),
        signature,
        universals: 0,
        assumptions: Vec::new(),
        parameters,
        type_assumptions,
        obligations: Vec::new(),
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
    })
}

fn output_interface(facts: &Declarations, contract: &mut Contract) -> Result<(), &'static str> {
    let source = contract
        .source
        .source()
        .ok_or("source output collector cannot populate an imported contract")?;
    contract.output = facts
        .nested_outputs
        .as_deref()
        .and_then(|outputs| outputs.get(&source))
        .cloned();
    if let Some(output) = &contract.output {
        contract.universals = contract.universals.max(output.universals);
    }
    Ok(())
}
