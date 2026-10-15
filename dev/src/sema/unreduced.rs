//! Sparse declaration types retain projection inputs independently of their normal forms.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{GenericArgs, TypeId, parameters::predicates::Predicate},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Types {
    data: Option<Box<Data>>,
}
#[derive(Default)]
struct Data {
    nodes: HashMap<NodeKey, TypeId>,
    definitions: HashMap<DefId, TypeId>,
    arguments: HashMap<NodeKey, GenericArgs>,
    defaults: HashMap<DefId, TypeId>,
    predicates: HashMap<NodeKey, Predicate>,
}
impl Types {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }
    #[must_use]
    pub fn node(&self, key: NodeKey) -> Option<TypeId> {
        self.data.as_ref()?.nodes.get(&key).copied()
    }
    #[must_use]
    pub fn definition(&self, owner: DefId) -> Option<TypeId> {
        self.data.as_ref()?.definitions.get(&owner).copied()
    }
    #[must_use]
    pub fn arguments(&self, key: NodeKey) -> Option<GenericArgs> {
        self.data.as_ref()?.arguments.get(&key).copied()
    }
    #[must_use]
    pub fn default_type(&self, parameter: DefId) -> Option<TypeId> {
        self.data.as_ref()?.defaults.get(&parameter).copied()
    }
    #[must_use]
    pub fn predicate(&self, key: NodeKey) -> Option<Predicate> {
        self.data.as_ref()?.predicates.get(&key).copied()
    }
    pub(super) fn retain_predicate(&mut self, key: NodeKey, predicate: Predicate) {
        self.data
            .get_or_insert_with(Default::default)
            .predicates
            .entry(key)
            .or_insert(predicate);
    }
    pub(super) fn retain_node(&mut self, key: NodeKey, ty: TypeId) {
        self.data
            .get_or_insert_with(Default::default)
            .nodes
            .entry(key)
            .or_insert(ty);
    }
    pub(super) fn retain_definition(&mut self, owner: DefId, ty: TypeId) {
        self.data
            .get_or_insert_with(Default::default)
            .definitions
            .insert(owner, ty);
    }
    pub(super) fn retain_arguments(&mut self, key: NodeKey, arguments: GenericArgs) {
        self.data
            .get_or_insert_with(Default::default)
            .arguments
            .insert(key, arguments);
    }
    pub(super) fn retain_default(&mut self, parameter: DefId, ty: TypeId) {
        self.data
            .get_or_insert_with(Default::default)
            .defaults
            .insert(parameter, ty);
    }
}
