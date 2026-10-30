//! Concrete trait callables select checked overrides while retaining their lifetime proof.
mod allocation;
mod arguments;
pub(in crate::sema) use arguments::callable_substitution;
pub(in crate::sema) use arguments::instantiate_charged as instantiate_callable;
mod consteval_defaults;
mod dependencies;
mod effect_requirements;
pub(crate) use consteval_defaults::check_const_default;
mod observed_constants;
mod proven_methods;
pub(in crate::sema) use observed_constants::{ConstantDecision, ConstantProof};
pub(in crate::sema) use proven_methods::selected_definition;
mod implementation;
mod non_type_callers;
mod objects;
mod reservations;
mod tables;
pub(crate) use super::traits::obligations::Obligations;
pub(crate) use super::traits::selection::Dependency as PreparationDependency;
use super::{
    model::Declarations,
    traits::{assumptions::Assumptions, selection},
};
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    types::{
        Kind, RegionId, TypeId, infer::Context, parameters::environment::Environment,
        substitute::Bound,
    },
};
pub(crate) use dependencies::Dependencies;
pub(crate) use objects::object_method;
use std::sync::Arc;
pub(crate) use tables::table_method;

/// A borrowed selected producer supplies data for a suspended query, never proof.
pub(crate) trait Preparation {
    fn with_original(
        &mut self,
        facts: &mut Declarations,
        definition: DefId,
        consume: &mut dyn FnMut(
            &Declarations,
            &crate::sema::crates::SourceOriginal<'_>,
        ) -> Result<(), crate::resolve::package::Error>,
    ) -> Result<(), crate::resolve::package::Error>;
    fn copy_declarations(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<(), crate::resolve::package::Error>;
    fn prepay(
        &mut self,
        facts: &mut Declarations,
        work: usize,
        bytes: usize,
    ) -> Result<(), crate::resolve::package::Error>;
    fn prepare(
        &mut self,
        facts: &mut Declarations,
        dependency: PreparationDependency,
    ) -> Result<(), crate::resolve::package::Error>;
}

pub(crate) enum Failure {
    Invalid(String),
    Preparation(crate::resolve::package::Error),
}
impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}
impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        Self::Invalid(message.into())
    }
}
impl Failure {
    pub(crate) fn message(self) -> String {
        match self {
            Self::Invalid(message) => message,
            Self::Preparation(error) => error.to_string(),
        }
    }
}

struct Caller {
    shared_environment: Option<Arc<Environment>>,
    environment: Option<Arc<Environment>>,
    assumptions: Assumptions,
}

enum Expansion {
    Active,
    Complete { ready: bool },
}

pub(crate) struct Dispatcher {
    availability: super::traits::implementations::Availability,
    share_constants: bool,
    provisional_size: bool,
    normalize: Bound,
    context: Context,
    obligations: Obligations,
    selected: HashMap<TypeId, TypeId>,
    constants: HashMap<super::values::Instance, super::values::Instance>,
    expanded: HashMap<super::values::Instance, Expansion>,
    caller: Option<Box<Caller>>,
    retained: Option<Box<allocation::Retained>>,
}
impl Dispatcher {
    pub(crate) fn new(universals: u32, first: u32) -> Result<Self, &'static str> {
        let end = first
            .checked_add(universals)
            .filter(|&end| end < 0x8000_0000)
            .ok_or("dispatch region identity space exhausted")?;
        if first == 0 {
            return Err("dispatch requires a free region allocator");
        }
        let mut obligations = Obligations::new(first);
        obligations.regions.end = end;
        let regions = (first..end).map(RegionId).collect::<Vec<_>>();
        for (index, &region) in regions.iter().enumerate() {
            let slot = u16::try_from(index).map_err(|_| "too many dispatch universals")?;
            obligations
                .equalities
                .push((region, RegionId::bound(0, slot)?));
        }
        Ok(Self {
            availability: super::traits::implementations::Availability::Validated,
            share_constants: universals == 0,
            provisional_size: false,
            normalize: Bound::new(regions),
            context: Context::default(),
            obligations,
            selected: HashMap::default(),
            constants: HashMap::default(),
            expanded: HashMap::default(),
            caller: None,
            retained: None,
        })
    }
    pub(in crate::sema) fn declaration(first: u32) -> Result<Self, &'static str> {
        let mut result = Self::new(0, first)?;
        result.availability = super::traits::implementations::Availability::Declarations;
        Ok(result)
    }

    /// Request a checked member without publishing a value or completing its impl.
    pub(in crate::sema) fn constant_owner(
        &mut self,
        facts: &mut Declarations,
        instance: super::values::Instance,
    ) -> Result<Option<DefId>, String> {
        let Some(owner) = facts.associated.trait_owner(instance.definition) else {
            return Ok(None);
        };
        let predicate = arguments::Arguments::constant(facts, instance)?.goal(facts, owner)?;
        let (owner, _, evidence) = self.implementation(facts, predicate)?;
        self.extend_evidence(evidence)?;
        Ok(Some(owner))
    }
    /// Patterns must select a definition before monomorphization. Rigid caller
    /// parameters may remain in unused arguments, but an assumption is not a value.
    pub(crate) fn pattern(
        facts: &mut Declarations,
        universals: u32,
        first: u32,
        owner: Option<DefId>,
    ) -> Result<Self, String> {
        let mut result = Self::new(universals, first)?;
        let shared_environment = owner
            .and_then(|owner| facts.generics.environments.get(&owner))
            .cloned();
        let environment = shared_environment
            .as_ref()
            .map(|environment| {
                environment
                    .substitute_regions(&mut facts.types, &mut result.normalize)
                    .map(Arc::new)
            })
            .transpose()?;
        result
            .context
            .account_work(result.normalize.steps)
            .map_err(|e| e.to_string())?;
        let mut assumptions = Assumptions::default();
        let clauses = environment
            .as_ref()
            .map(|env| env.trait_clauses())
            .transpose()?;
        for clause in clauses.into_iter().flatten() {
            result.context.account_work(1).map_err(|e| e.to_string())?;
            // Only a fully global bound can use its known implementation.
            // Free lifetimes include 'static; either that input or a parameter
            // keeps the caller assumption opaque to constant-value selection.
            let opaque = match clause {
                crate::types::parameters::clauses::Clause::Predicate(predicate) => {
                    predicate.contains_parameters(&facts.types)?
                        || predicate.has_free_regions(&facts.types)?
                }
                crate::types::parameters::clauses::Clause::ForAll(_)
                | crate::types::parameters::clauses::Clause::UnstableFeature(_) => true,
            };
            if opaque {
                assumptions
                    .insert_clause(&facts.types, clause)
                    .map_err(|e| e.to_string())?;
            }
        }
        result.caller = Some(Box::new(Caller {
            shared_environment,
            environment,
            assumptions,
        }));
        Ok(result)
    }
    pub(crate) fn select(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<TypeId, String> {
        self.select_using(facts, ty, None).map_err(Failure::message)
    }
    pub(crate) fn select_prepared(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        preparation: &mut dyn Preparation,
    ) -> Result<TypeId, Failure> {
        self.select_using(facts, ty, Some(preparation))
    }
    fn select_using(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        mut preparation: Option<&mut dyn Preparation>,
    ) -> Result<TypeId, Failure> {
        if let Some(actual) = &mut preparation {
            actual.prepay(facts, 1, 0).map_err(Failure::Preparation)?;
        }
        if let Some(&selected) = self.selected.get(&ty) {
            return Ok(selected);
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
            return Err("dispatch requires a function item".into());
        };
        let method = DefId(item.0);
        let Some(trait_owner) = facts.associated.trait_owner(method) else {
            return Ok(ty);
        };
        if facts.types.contains_parameters(ty) != Some(false) {
            return Err("concrete dispatch retains unresolved parameters".into());
        }
        let arguments = arguments::Arguments::new(facts, ty)?;
        let mut predicate = arguments.goal(facts, trait_owner)?;
        if self.caller.is_none()
            && matches!(
                self.availability,
                super::traits::implementations::Availability::Validated
            )
            && matches!(facts.types.kind(predicate.ty), Some(Kind::Opaque(..)))
        {
            facts.prepare_opaque_storage()?;
            predicate.ty = facts.types.storage_head(predicate.ty)?;
        }
        let (owner, implementation, evidence) =
            match self.method_implementation_using(facts, predicate, preparation)? {
                objects::Method::Implementation {
                    owner,
                    arguments,
                    proof,
                } => (owner, arguments, proof),
                objects::Method::Object { proof } => {
                    self.extend_evidence(proof)?;
                    self.selected.insert(ty, ty);
                    return Ok(ty);
                }
                objects::Method::Language { descriptor, proof } => {
                    objects::retain_language(facts, method, ty, descriptor)?;
                    self.extend_evidence(proof)?;
                    self.selected.insert(ty, ty);
                    return Ok(ty);
                }
            };
        let selected =
            super::specialization::selected_method(facts, owner, implementation, method)?;
        let target = selected.definition();
        let implementation = selected.implementation();
        self.extend_evidence(evidence)?;
        let mut result = if target == method {
            ty
        } else {
            let (owner, implementation) = implementation
                .ok_or("selected override lacks its actual defining implementation")?;
            arguments.target(facts, owner, implementation, target)?
        };
        if facts.types.contains_projections(result) == Some(true) {
            // Selecting an override can introduce its written projections after
            // the original operand was normalized. Keep their formation debts
            // in this dispatch proof, including nested callable scopes.
            result = self.normalize_override(facts, result)?;
        }
        self.selected.insert(ty, result);
        Ok(result)
    }
    pub(crate) fn constant(
        &mut self,
        facts: &mut Declarations,
        instance: super::values::Instance,
    ) -> Result<super::values::Instance, String> {
        self.constant_using::<false>(facts, instance, &mut |_, _| {
            Ok(ConstantDecision::Checked(None))
        })
    }
    /// Observe only newly selected evidence; the ordinary route has no observer
    /// and retains its existing cache and publication behavior.
    pub(in crate::sema) fn source_constant(
        &mut self,
        facts: &mut Declarations,
        instance: super::values::Instance,
        observer: &mut impl FnMut(
            &mut Declarations,
            ConstantProof<'_>,
        ) -> Result<ConstantDecision, String>,
    ) -> Result<super::values::Instance, String> {
        self.constant_using::<true>(facts, instance, observer)
    }
    fn constant_using<const OBSERVE: bool>(
        &mut self,
        facts: &mut Declarations,
        instance: super::values::Instance,
        observer: &mut impl FnMut(
            &mut Declarations,
            ConstantProof<'_>,
        ) -> Result<ConstantDecision, String>,
    ) -> Result<super::values::Instance, String> {
        let selected = self.select_constant_using::<OBSERVE>(facts, instance, observer)?;
        facts.constant_targets.insert(instance, selected);
        let share = self.share_constants && self.caller.is_none();
        let mut incomplete = 0;
        let mut work = vec![(selected, None)];
        while let Some((current, finish)) = work.pop() {
            self.context.account_work(1).map_err(|e| e.to_string())?;
            if let Some(before) = finish {
                self.expanded.insert(
                    current,
                    Expansion::Complete {
                        ready: before == incomplete,
                    },
                );
                // Missing typed dependencies are not an empty completed graph.
                // An empty proof permits sharing across callers only when no
                // caller environment or universal mapping participated.
                if share
                    && !self.provisional_size
                    && before == incomplete
                    && self.obligations.is_empty()
                    && self.context.region_equalities.is_empty()
                    && facts.types.arguments_are_concrete(current.arguments)?
                {
                    facts
                        .dispatch_dependencies
                        .insert(self.availability, current);
                }
                continue;
            }
            if share
                && facts
                    .dispatch_dependencies
                    .contains(self.availability, current)
            {
                self.expanded
                    .insert(current, Expansion::Complete { ready: true });
                continue;
            }
            match self.expanded.get(&current) {
                Some(Expansion::Complete { ready }) => {
                    incomplete += usize::from(!ready);
                    continue;
                }
                Some(Expansion::Active) => {
                    return Err("cycle in associated constant dependencies".into());
                }
                None => {}
            }
            self.expanded.insert(current, Expansion::Active);
            facts.dispatch_dependencies.work.dependency_expansions += 1;
            if self.expanded.len() > 1_000_000 {
                return Err("constant selection dependency budget exceeded".into());
            }
            work.push((current, Some(incomplete)));
            if !facts.dispatch_dependencies.ready(current.definition) {
                incomplete += 1;
            }
            let Some(dependencies) = facts
                .constant_dependencies
                .get(&current.definition)
                .cloned()
            else {
                continue;
            };
            self.context
                .account_work(dependencies.len())
                .map_err(|e| e.to_string())?;
            let mut substitution = current.substitution(facts)?;
            for dependency in dependencies.into_iter().rev() {
                let before = substitution.steps;
                match dependency {
                    super::values::Dependency::Constant(instance) => {
                        let source = instance.substitute(&mut facts.types, &mut substitution)?;
                        let target =
                            self.select_constant_using::<OBSERVE>(facts, source, observer)?;
                        facts.constant_targets.insert(source, target);
                        work.push((target, None));
                    }
                    super::values::Dependency::Callable(ty) => {
                        let source = substitution.apply(&mut facts.types, ty)?;
                        let target = self.select(facts, source)?;
                        facts
                            .generics
                            .constant_callable_targets
                            .insert(source, target);
                    }
                }
                self.context
                    .account_work(substitution.steps - before)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(selected)
    }
    fn implementation(
        &mut self,
        facts: &mut Declarations,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(DefId, crate::types::GenericArgs, Obligations), String> {
        self.implementation_using(facts, predicate, None)
            .map_err(Failure::message)
    }
    fn implementation_using(
        &mut self,
        facts: &mut Declarations,
        predicate: crate::types::parameters::predicates::Predicate,
        preparation: Option<&mut dyn Preparation>,
    ) -> Result<(DefId, crate::types::GenericArgs, Obligations), Failure> {
        let availability = if preparation.is_some() {
            super::traits::implementations::Availability::Executable
        } else {
            self.availability
        };
        let before = self.normalize.steps;
        let predicate = self.normalize.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(self.normalize.steps - before + 1)
            .map_err(|e| e.to_string())?;
        let closed = self.share_constants
            && self.caller.is_none()
            && facts
                .generics
                .implementations
                .as_ref()
                .map(|registry| {
                    registry.selection_predicate_ready(&facts.types, &mut self.context, predicate)
                })
                .transpose()
                .map_err(|e| e.to_string())?
                .unwrap_or(false)
            && predicate.is_concrete(&facts.types)?;
        let first = self.obligations.regions.end;
        if closed
            && let Some(target) = facts
                .dispatch_dependencies
                .selection(availability, predicate)
        {
            facts.dispatch_dependencies.work.shared_selections += 1;
            return Ok((target.owner, target.arguments, Obligations::new(first)));
        }
        let empty = Assumptions::default();
        let caller = self.caller.as_ref();
        let caller = selection::Caller {
            assumptions: caller.map_or(&empty, |c| &c.assumptions),
            environment: caller.and_then(|c| c.environment.as_ref()),
            first_region: first,
            retain_sized: false,
        };
        let (target, proof) = if closed {
            // No inference term or private identity from a discharged closed
            // query survives into this dispatcher's other selections.
            let mut context = Context::default();
            let (target, proof, provisional) = implementation::select_using(
                facts,
                &mut context,
                predicate,
                caller,
                availability,
                preparation,
            )?;
            self.provisional_size |= provisional;
            let certified = implementation::certify(facts, &mut context, target, &proof)?;
            self.context
                .account_work(context.steps)
                .map_err(|e| e.to_string())?;
            if let Some(target) = certified {
                if provisional {
                    facts
                        .generics
                        .size_rules
                        .get_or_insert_with(Default::default)
                        .require_dispatch(super::nominals::sizes::Dispatch {
                            predicate,
                            owner: target.owner,
                            arguments: target.arguments,
                        })?;
                } else {
                    facts
                        .dispatch_dependencies
                        .certify(availability, predicate, target);
                }
                (target, Obligations::new(first))
            } else {
                (target, proof)
            }
        } else {
            let (target, proof, provisional) = implementation::select_using(
                facts,
                &mut self.context,
                predicate,
                caller,
                availability,
                preparation,
            )?;
            self.provisional_size |= provisional;
            (target, proof)
        };
        Ok((target.owner, target.arguments, proof))
    }

    /// An independently discharged closed proof can be repeated without a body
    /// namespace. Its chosen target must remain identical after sizes settle.
    pub(in crate::sema) fn recheck_size(
        facts: &mut Declarations,
        expected: super::nominals::sizes::Dispatch,
    ) -> Result<(), String> {
        let mut dispatch = Self::declaration(1)?;
        let (owner, arguments, proof) = dispatch.implementation(facts, expected.predicate)?;
        if owner != expected.owner || arguments != expected.arguments || !proof.is_empty() {
            return Err("constant selection changed after nominal sizedness completed".into());
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Obligations {
        self.obligations
            .equalities
            .append(&mut self.context.region_equalities);
        self.obligations
    }
}
