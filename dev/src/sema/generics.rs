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

#[derive(Clone, Copy)]
pub struct Parameter {
    pub binding: DefId,
    pub argument: Argument,
    pub default: Option<Argument>,
}
#[derive(Default)]
pub struct Generics {
    pub parameters: HashMap<DefId, Vec<Parameter>>,
    pub constants: HashMap<DefId, ConstId>,
    pub environments: HashMap<DefId, Arc<Environment>>,
    substitutions: HashMap<(DefId, GenericArgs), Bound>,
}
impl Generics {
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
        let Some(parameters) = self.parameters.get(&owner) else {
            return Ok(());
        };
        let mut assumptions = Vec::new();
        for parameter in parameters {
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
                entry.insert(Arc::new(Environment::new(store, &assumptions)?));
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
