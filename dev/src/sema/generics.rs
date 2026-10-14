//! Completed declaration environments and substitutions keyed by owner and arguments.
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, Kind, Store, TypeId,
        arguments::{Argument, ConstId},
        parameters::environment::{Abilities, Environment},
        substitute::Bound,
    },
};
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

impl super::model::Declarations {
    /// Read a resolved const parameter through the permitted argument wrappers.
    pub(super) fn const_parameter(
        &self,
        report: &crate::resolve::expansion::Report,
        mut key: crate::resolve::unit::NodeKey,
    ) -> Option<ConstId> {
        use crate::syntax::ast::{BlockFlavor, Kind, TypeFlavor};
        let mut block = false;
        loop {
            let ast = &report.package.unit(key.file).configured.ast;
            match ast.node(key.node).kind {
                Kind::Block {
                    flavor: BlockFlavor::Plain,
                    statements,
                    tail: Some(node),
                } if statements.start == statements.end && !block => {
                    block = true;
                    key.node = node;
                }
                Kind::Type {
                    flavor: TypeFlavor::Path,
                    elements,
                } => {
                    let nodes = ast.list(elements);
                    if nodes.len() != 1 {
                        return None;
                    }
                    key.node = nodes[0];
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
    pub argument: Argument,
    pub default: Option<Argument>,
}
#[derive(Clone, Copy)]
pub enum SourceArgument {
    Type(crate::resolve::unit::NodeKey),
    Const(ConstId),
    Infer,
}
pub struct FunctionUse {
    pub lifetimes: Option<crate::syntax::ast::List>,
    pub arguments: Vec<SourceArgument>,
}
#[derive(Clone, Copy)]
pub struct Predicate {
    pub source: crate::resolve::unit::NodeKey,
    pub value: crate::types::parameters::predicates::Predicate,
}
#[derive(Default)]
pub struct Generics {
    pub predicates: HashMap<DefId, Vec<Predicate>>,
    pub parameters: HashMap<DefId, Vec<Parameter>>,
    pub constants: HashMap<DefId, ConstId>,
    pub environments: HashMap<DefId, Arc<Environment>>,
    /// Sparse overrides for free universal aliases used by completed body facts.
    pub(super) body_environments: HashMap<DefId, Arc<Environment>>,
    pub function_uses: HashMap<crate::resolve::unit::NodeKey, FunctionUse>,
    substitutions: HashMap<(DefId, GenericArgs), Bound>,
}
impl Generics {
    #[must_use]
    pub fn body_environment(&self, owner: DefId) -> Option<&Arc<Environment>> {
        self.body_environments
            .get(&owner)
            .or_else(|| self.environments.get(&owner))
    }
    pub(super) fn bound_lifetimes(
        &self,
        types: &Store,
        owner: DefId,
    ) -> Result<std::collections::HashSet<u16>, &'static str> {
        let mut uses = crate::types::bound_uses::Uses::default();
        for predicate in self.predicates.get(&owner).into_iter().flatten() {
            uses.collect(types, predicate.value.ty)?;
            if let crate::types::parameters::predicates::Bound::Outlives(region) =
                predicate.value.bound
                && let Some((0, index)) = region.bound_parts()
            {
                uses.indices.insert(index);
            }
        }
        Ok(uses.indices)
    }
    pub(super) fn parameter_ids(
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
                Argument::Lifetime(_) => {
                    return Err("callable metadata contains a late lifetime slot");
                }
            };
            ids.push(id);
        }
        Ok(ids)
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
            .find(|f| self.parameters.contains_key(&f.definition))
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
        store: &Store,
        owner: DefId,
    ) -> Result<(), &'static str> {
        if !self.parameters.contains_key(&owner) && !self.predicates.contains_key(&owner) {
            return Ok(());
        }
        let mut assumptions = Vec::new();
        for parameter in self.parameters.get(&owner).into_iter().flatten() {
            if let Argument::Type(ty) = parameter.argument {
                let Some(Kind::Parameter(id)) = store.kind(ty) else {
                    return Err("generic declaration requires a rigid type parameter");
                };
                assumptions.push((id, Abilities::default()));
            }
        }
        match self.environments.entry(owner) {
            Entry::Occupied(_) => Err("generic body environment already completed"),
            Entry::Vacant(entry) => {
                let predicates = self
                    .predicates
                    .get(&owner)
                    .into_iter()
                    .flatten()
                    .map(|p| p.value)
                    .collect::<Vec<_>>();
                entry.insert(Arc::new(
                    Environment::new(store, &assumptions)?.with_predicates(store, &predicates)?,
                ));
                Ok(())
            }
        }
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
    /// Defaults are templates in the same declaration environment as their owner.
    /// # Errors
    /// Rejects invalid earlier arguments and exhausted substitution resources.
    pub fn default_argument(
        &mut self,
        store: &mut Store,
        owner: DefId,
        args: GenericArgs,
        argument: Argument,
    ) -> Result<Argument, &'static str> {
        self.substitution(store, owner, args)?
            .apply_argument(store, argument)
    }
}
