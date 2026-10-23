//! Completed declaration environments and substitutions keyed by owner and arguments.
mod builtin_formation;
pub(super) mod captures;
mod families;
mod input_contracts;
mod traits;
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, Kind, Store, TypeId,
        arguments::{Argument, ConstId},
        parameters::environment::{Abilities, Environment},
        substitute::Bound,
    },
};
use std::{collections::hash_map::Entry, sync::Arc};

pub(super) use crate::types::parameters::argument_id as parameter_id;

impl super::model::Declarations {
    /// Read a resolved const parameter through the permitted argument wrappers.
    pub(super) fn const_parameter(
        &self,
        report: &crate::resolve::expansion::Report,
        mut key: crate::resolve::unit::NodeKey,
    ) -> Option<ConstId> {
        use crate::syntax::ast::{BlockFlavor, Kind};
        let mut block = false;
        loop {
            let ast = &report.package.unit(key.file).configured.ast;
            match ast.node(key.node).kind {
                Kind::Block {
                    flavor: BlockFlavor::Plain | BlockFlavor::Unsafe,
                    statements,
                    tail: Some(node),
                } if statements.start == statements.end && !block => {
                    block = true;
                    key.node = node;
                }
                _ => break,
            }
        }
        let crate::resolve::model::Target::Definition(id) = self.node(key)?.reference? else {
            return None;
        };
        self.generics.constants.get(&id).copied()
    }
}

#[derive(Clone, Copy)]
pub struct Parameter {
    pub binding: DefId,
    pub anonymous: bool,
    pub argument: Argument,
    pub default: Option<Argument>,
}
#[derive(Clone, Copy)]
pub enum SourceArgument {
    Type(crate::resolve::unit::NodeKey),
    Const(ConstId),
    Infer,
}
pub struct ItemUse {
    pub lifetimes: Option<crate::syntax::ast::List>,
    pub arguments: Vec<SourceArgument>,
}
/// Constructor arguments come from a separately typed path or an impl's exact Self.
#[derive(Clone, Copy)]
pub enum ConstructorUse {
    Prefix(crate::resolve::unit::NodeKey),
    SelfType(TypeId),
}
pub(super) struct TraitUse {
    pub owner: DefId,
    /// A written trait prefix supplies arguments and type-path defaults.
    pub arguments: Option<ItemUse>,
    /// The written Self type in either a fully qualified or implicit trait path.
    pub subject: Option<crate::resolve::unit::NodeKey>,
}
#[derive(Clone, Copy)]
pub(super) enum DeferredArgument {
    Type(crate::resolve::unit::NodeKey),
    Const(crate::resolve::unit::NodeKey),
    Infer,
}
pub(super) struct DeferredUse {
    pub lifetimes: Option<crate::syntax::ast::List>,
    pub arguments: Vec<DeferredArgument>,
}
#[derive(Clone, Copy)]
pub enum AnnotationArgument {
    Lifetime(crate::types::RegionId),
    Type(crate::resolve::unit::NodeKey),
    Const(ConstId),
    Infer,
    Default(Argument),
}
pub struct AnnotationUse {
    pub owner: DefId,
    pub arguments: Vec<AnnotationArgument>,
}
/// Resolved identities for a qualified local annotation whose inputs have holes.
#[derive(Clone, Copy)]
pub(super) struct AnnotationProjection {
    pub subject: crate::resolve::unit::NodeKey,
    pub parent: crate::resolve::unit::NodeKey,
    pub member: crate::resolve::unit::NodeKey,
    pub declaration: DefId,
}
#[derive(Clone, Copy)]
pub struct Predicate {
    pub source: crate::resolve::unit::NodeKey,
    pub value: crate::types::parameters::clauses::Clause,
}
#[derive(Default)]
pub struct Generics {
    pub(crate) builtin_requirements:
        HashMap<crate::resolve::unit::NodeKey, super::model::BuiltinRequirements>,
    builtin_formations: Option<Box<builtin_formation::Builtins>>,
    /// Concrete initializer callables consumed by the shared constant value reader.
    pub(crate) constant_callable_targets: HashMap<TypeId, TypeId>,
    pub(super) trait_uses: HashMap<crate::resolve::unit::NodeKey, TraitUse>,
    /// Exact implicit Self: Trait<parameters> predicate, before callable substitution.
    pub(super) trait_self_bounds: HashMap<DefId, crate::types::parameters::predicates::Predicate>,
    /// Selected impl evidence at body bound uses, after proving its conditions.
    pub trait_selections: HashMap<crate::resolve::unit::NodeKey, Vec<ImplementationSelection>>,
    pub(super) implementations: Option<Box<super::traits::implementations::Implementations>>,
    pub(super) traits: Option<Box<super::traits::Contracts>>,
    pub(super) method_uses: HashMap<crate::resolve::unit::NodeKey, DeferredUse>,
    pub(super) parents: HashMap<DefId, DefId>,
    pub storage: Option<crate::types::aggregate::finiteness::Storage>,
    pub(super) size_rules: Option<Box<super::nominals::sizes::Pending>>,
    pub(super) nominal_contracts: crate::support::hash::HashSet<DefId>,
    pub(super) nominal_requirements:
        HashMap<TypeId, Arc<[crate::types::parameters::clauses::Clause]>>,
    pub(super) has_nominal_requirements: bool,
    /// Crate-wide answers for closed body goals in neutral callers.
    pub(super) selection_memo: super::traits::selection::Memo,
    pub predicates: HashMap<DefId, Vec<Predicate>>,
    /// Prefix before supertrait elaboration. Implied assumptions are not additional
    /// impl selection conditions: re-proving them can manufacture a circular goal.
    declared_predicate_counts: HashMap<DefId, usize>,
    pub parameters: HashMap<DefId, Vec<Parameter>>,
    pub constants: HashMap<DefId, ConstId>,
    pub environments: HashMap<DefId, Arc<Environment>>,
    pub(super) projection_promises:
        Option<Arc<HashMap<ItemId, crate::types::projection::Builtins>>>,
    /// Sparse overrides for free universal aliases used by completed body facts.
    pub(super) body_environments: HashMap<DefId, Arc<Environment>>,
    pub item_uses: HashMap<crate::resolve::unit::NodeKey, ItemUse>,
    pub constructor_uses: HashMap<crate::resolve::unit::NodeKey, ConstructorUse>,
    /// Implicit callable factories selected while checking expressions. Nominal
    /// declarations alone do not require synthetic field-transfer bodies.
    pub(crate) referenced_constructors: crate::support::hash::HashSet<ItemId>,
    pub annotation_uses: HashMap<crate::resolve::unit::NodeKey, AnnotationUse>,
    pub(super) annotation_projections: HashMap<crate::resolve::unit::NodeKey, AnnotationProjection>,
    substitutions: HashMap<(DefId, GenericArgs), Bound>,
    family_substitutions: Option<Box<families::Substitutions>>,
}
pub struct ImplementationSelection {
    pub owner: DefId,
    pub arguments: GenericArgs,
}
impl Generics {
    #[cfg(test)]
    pub(in crate::sema) fn substitution_steps(&self) -> usize {
        self.substitutions.values().map(|fold| fold.steps).sum()
    }

    /// Only completed conformance records may authorize trait selection or dispatch.
    pub fn implementations(
        &self,
    ) -> impl Iterator<Item = &super::traits::implementations::Implementation> {
        self.implementations
            .iter()
            .flat_map(|registry| registry.validated())
    }
    fn scopes(&self, owner: DefId) -> impl Iterator<Item = DefId> + '_ {
        std::iter::successors(Some(owner), |id| self.parents.get(id).copied())
    }
    /// Parameters retain their declaration identities when inherited by a method.
    pub fn parameters_in_scope(&self, owner: DefId) -> impl Iterator<Item = &Parameter> {
        self.scopes(owner)
            .flat_map(|id| self.parameters.get(&id).into_iter().flatten())
    }
    pub fn predicates_in_scope(&self, owner: DefId) -> impl Iterator<Item = &Predicate> {
        self.scopes(owner)
            .flat_map(|id| self.predicates.get(&id).into_iter().flatten())
    }
    pub(super) fn declared_predicates_in_scope(
        &self,
        owner: DefId,
    ) -> impl Iterator<Item = &Predicate> {
        self.scopes(owner)
            .flat_map(|id| self.declared_predicates(id))
    }
    pub(super) fn declared_predicates(&self, owner: DefId) -> &[Predicate] {
        let predicates = self.predicates.get(&owner).map_or(&[][..], Vec::as_slice);
        let count = self
            .declared_predicate_counts
            .get(&owner)
            .copied()
            .unwrap_or(predicates.len());
        &predicates[..count]
    }
    #[must_use]
    pub fn has_parameters(&self, owner: DefId) -> bool {
        self.scopes(owner)
            .any(|id| self.parameters.contains_key(&id))
    }
    #[must_use]
    pub fn body_environment(&self, owner: DefId) -> Option<&Arc<Environment>> {
        self.body_environments
            .get(&owner)
            .or_else(|| self.environments.get(&owner))
    }
    pub(super) fn bound_lifetimes(
        &self,
        types: &Store,
        originals: &super::unreduced::Types,
        owner: DefId,
    ) -> Result<crate::support::hash::HashSet<u16>, &'static str> {
        let mut uses = crate::types::bound_uses::Uses::default();
        for predicate in self.predicates.get(&owner).into_iter().flatten() {
            uses.clause(types, predicate.value)?;
            if let Some(original) = originals.predicate(predicate.source) {
                uses.clause(types, original)?;
            }
        }
        Ok(uses.indices)
    }
    pub(crate) fn parameter_ids(
        &self,
        store: &Store,
        owner: DefId,
    ) -> Result<Vec<crate::types::parameters::ParameterId>, &'static str> {
        let mut ids = Vec::new();
        for parameter in self.parameters.get(&owner).into_iter().flatten() {
            let id = match parameter.argument {
                Argument::Type(ty) => match store.kind(ty) {
                    Some(Kind::Parameter(id)) => id,
                    _ => return Err("missing rigid type parameter"),
                },
                Argument::Const(id) => match store.const_kind(id) {
                    Some(crate::types::arguments::ConstKind::Parameter(id)) => id,
                    _ => return Err("missing rigid const parameter"),
                },
                Argument::Lifetime(_) => continue,
            };
            ids.push(id);
        }
        Ok(ids)
    }
    pub(super) fn callable_parameters(
        &self,
        store: &Store,
        owner: DefId,
    ) -> Result<(Vec<ItemId>, Vec<crate::types::parameters::ParameterId>), &'static str> {
        let mut owners: Vec<_> = self.scopes(owner).collect();
        owners.reverse();
        let mut parameters = Vec::new();
        for &owner in &owners {
            parameters.extend(self.parameter_ids(store, owner)?);
        }
        Ok((
            owners.into_iter().map(|id| ItemId(id.0)).collect(),
            parameters,
        ))
    }
    /// Until generic move/drop plans exist, symbolic bodies must not enter the
    /// concrete CFG builder, which can discard temporaries that appear dropless.
    /// # Errors
    /// Reports the first function requiring generic body construction.
    pub fn require_concrete_bodies(
        &self,
        report: &crate::resolve::expansion::Report,
        functions: &[super::model::Function],
    ) -> Result<(), crate::resolve::package::Error> {
        if let Some(function) = functions
            .iter()
            .find(|f| f.body.is_some() && self.has_parameters(f.definition))
        {
            return Err(super::model::Declarations::error(
                report,
                function.node,
                "generic body construction requires parameterized ownership and destruction",
            ));
        }
        Ok(())
    }
    /// Freeze the assumptions before checking a function's symbolic body.
    /// # Errors
    /// Rejects malformed parameter facts and repeated publication.
    pub fn complete_environment(
        &mut self,
        store: &mut Store,
        owner: DefId,
    ) -> Result<(), &'static str> {
        if !self
            .scopes(owner)
            .any(|id| self.parameters.contains_key(&id) || self.predicates.contains_key(&id))
        {
            return Ok(());
        }
        let environment = self.elaborated_environment(store, owner)?;
        match self.environments.entry(owner) {
            Entry::Occupied(_) => Err("generic body environment already completed"),
            Entry::Vacant(entry) => {
                entry.insert(Arc::new(environment));
                Ok(())
            }
        }
    }
    pub(crate) fn elaborated_environment(
        &mut self,
        store: &mut Store,
        owner: DefId,
    ) -> Result<Environment, &'static str> {
        self.elaborate_trait_bounds(store, owner)?;
        self.parameter_environment(store, owner)
    }
    /// Build assumptions from completed declaration parameters and predicates.
    /// Consumers may share this immutable environment across implicit bodies.
    /// # Errors
    /// Rejects malformed parameter identities and predicate subjects.
    pub fn parameter_environment(
        &self,
        store: &Store,
        owner: DefId,
    ) -> Result<Environment, &'static str> {
        let predicates = self
            .predicates_in_scope(owner)
            .map(|p| p.value)
            .collect::<Vec<_>>();
        self.parameter_scope(store, owner)?
            .with_clauses(store, &predicates)
    }
    /// Declare parameters without granting the unenforced written bounds of an alias.
    /// # Errors
    /// Rejects malformed parameter identities.
    pub(crate) fn parameter_scope(
        &self,
        store: &Store,
        owner: DefId,
    ) -> Result<Environment, &'static str> {
        self.declared_scope(store, self.parameters_in_scope(owner).copied())
    }
    /// Conformance chooses rigid identities independently of an owner's bounds.
    pub(in crate::sema) fn declared_scope(
        &self,
        store: &Store,
        parameters: impl Iterator<Item = Parameter>,
    ) -> Result<Environment, &'static str> {
        let mut assumptions = Vec::new();
        let mut constants = Vec::new();
        for parameter in parameters {
            if let Argument::Type(ty) = parameter.argument {
                let Some(Kind::Parameter(id)) = store.kind(ty) else {
                    return Err("generic declaration requires a rigid type parameter");
                };
                assumptions.push((id, Abilities::default()));
            } else if let Argument::Const(id) = parameter.argument {
                let Some(crate::types::arguments::ConstKind::Parameter(id)) = store.const_kind(id)
                else {
                    return Err("generic declaration requires a rigid const parameter");
                };
                constants.push(id);
            }
        }
        Environment::new(store, &assumptions)?
            .with_projection_promises(self.projection_promises.clone())
            .with_const_parameters(store, &constants)
    }
    fn substitution(
        &mut self,
        store: &Store,
        owner: DefId,
        args: GenericArgs,
    ) -> Result<&mut Bound, &'static str> {
        Ok(match self.substitutions.entry((owner, args)) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Bound::joint(store, ItemId(owner.0), args)?),
        })
    }
    /// # Errors
    /// Rejects malformed identities, mismatched arguments and exhausted work.
    pub fn substitute(
        &mut self,
        store: &mut Store,
        owner: DefId,
        args: GenericArgs,
        ty: TypeId,
    ) -> Result<TypeId, &'static str> {
        self.substitution(store, owner, args)?.apply(store, ty)
    }
    /// Transform a bound's subject and arguments in the same cached parameter mapping.
    /// # Errors
    /// Rejects malformed identities, mismatched arguments and exhausted work.
    pub fn substitute_predicate(
        &mut self,
        store: &mut Store,
        owner: DefId,
        args: GenericArgs,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<crate::types::parameters::predicates::Predicate, &'static str> {
        self.substitution(store, owner, args)?
            .predicate(store, predicate)
    }
    /// Preserve a requirement's own binder while substituting the nominal's arguments.
    /// # Errors
    /// Rejects malformed identities, mismatched arguments and exhausted work.
    pub fn substitute_clause(
        &mut self,
        store: &mut Store,
        owner: DefId,
        args: GenericArgs,
        clause: crate::types::parameters::clauses::Clause,
    ) -> Result<crate::types::parameters::clauses::Clause, &'static str> {
        self.substitution(store, owner, args)?.clause(store, clause)
    }
    /// Retain the completed append-only environment built from these exact
    /// arguments. Subsequent body expansion reuses it just like an explicit use.
    pub(super) fn retain_arguments(
        &mut self,
        owner: DefId,
        args: GenericArgs,
        substitution: Bound,
    ) {
        self.substitutions
            .entry((owner, args))
            .or_insert(substitution);
    }
}
