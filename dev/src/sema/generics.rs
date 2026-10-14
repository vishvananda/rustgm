//! Completed declaration environments and substitutions keyed by owner and arguments.
use crate::{
    resolve::model::DefId,
    types::{
        GenericArgs, ItemId, Store, TypeId,
        arguments::{Argument, ConstId},
        substitute::Bound,
    },
};
use std::collections::{HashMap, hash_map::Entry};

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
    substitutions: HashMap<(DefId, GenericArgs), Bound>,
}
impl Generics {
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
