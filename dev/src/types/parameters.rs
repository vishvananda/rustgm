//! Immutable generic parameter declarations; inference variables have separate identities.
pub mod environment;
pub mod predicates;
pub mod shape;
pub mod uses;
use super::{ItemId, Kind, Primitive, Store, TypeId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterKind {
    /// Sizedness is a declaration requirement, not a concrete storage layout.
    Type {
        sized: bool,
    },
    Const {
        ty: TypeId,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parameter {
    pub owner: ItemId,
    /// Argument slot assigned by the declaration environment, not a source offset.
    pub index: u32,
    pub kind: ParameterKind,
}
#[derive(Default)]
pub(super) struct Storage {
    values: Vec<Parameter>,
    index: HashMap<(ItemId, u32), ParameterId>,
}
impl Store {
    #[must_use]
    pub fn parameter(&self, id: ParameterId) -> Option<Parameter> {
        self.arguments
            .as_ref()?
            .parameters
            .values
            .get(id.0 as usize)
            .copied()
    }
    /// Register one immutable parameter declaration. Repeating its declaration
    /// reuses its identity; changing its kind or requirements is an invariant error.
    ///
    /// # Errors
    /// Rejects conflicting declarations, invalid const parameter types and ID overflow.
    pub fn intern_parameter(&mut self, parameter: Parameter) -> Result<ParameterId, &'static str> {
        if let ParameterKind::Const { ty } = parameter.kind {
            let valid = matches!(self.kind(ty), Some(Kind::Primitive(p))
                if p.integer().is_some() || matches!(p, Primitive::Bool | Primitive::Char));
            if !valid {
                return Err("invalid const parameter type");
            }
        }
        let storage = &mut self
            .arguments
            .get_or_insert_with(Default::default)
            .parameters;
        let key = (parameter.owner, parameter.index);
        if let Some(id) = storage.index.get(&key) {
            return if storage.values[id.0 as usize] == parameter {
                Ok(*id)
            } else {
                Err("conflicting generic parameter declaration")
            };
        }
        let id = ParameterId(
            u32::try_from(storage.values.len()).map_err(|_| "too many generic parameters")?,
        );
        storage.values.push(parameter);
        storage.index.insert(key, id);
        Ok(id)
    }
}
