//! Immutable generic parameter declarations; inference variables have separate identities.
pub mod clauses;
pub(crate) mod constraints;
pub mod environment;
pub mod features;
pub mod predicates;
pub(crate) mod premise_index;
pub mod shape;
pub mod uses;
use super::{ItemId, Kind, Primitive, Store, TypeId};
use crate::support::hash::HashMap;

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
    /// Reserve the actual parameter-table growth independently of type nodes.
    pub(crate) fn parameter_allocation_bound(&self, count: usize) -> usize {
        let Some(arguments) = &self.arguments else {
            return size_of::<super::arguments::Storage>()
                .saturating_add(count.saturating_mul(4 * size_of::<Parameter>()))
                .saturating_add(crate::support::retained::new_map::<
                    (ItemId, u32),
                    ParameterId,
                >(count));
        };
        let storage = &arguments.parameters;
        let needed = storage.values.len().saturating_add(count);
        (if needed > storage.values.capacity() {
            needed.saturating_mul(4 * size_of::<Parameter>())
        } else {
            0
        })
        .saturating_add(
            if storage.index.len().saturating_add(count) > storage.index.capacity() {
                crate::support::retained::new_map::<(ItemId, u32), ParameterId>(
                    storage.index.len().saturating_add(count),
                )
            } else {
                0
            },
        )
    }
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

pub(crate) fn argument_id(
    store: &Store,
    formal: super::arguments::Argument,
) -> Result<crate::types::parameters::ParameterId, &'static str> {
    use super::arguments::Argument;
    match formal {
        Argument::Type(ty) => match store.kind(ty) {
            Some(Kind::Parameter(id)) => Ok(id),
            _ => Err("missing rigid type parameter"),
        },
        Argument::Const(constant) => match store.const_kind(constant) {
            Some(crate::types::arguments::ConstKind::Parameter(id)) => Ok(id),
            _ => Err("missing rigid const parameter"),
        },
        Argument::Lifetime(_) => Err("late lifetime was retained as a type argument"),
    }
}
