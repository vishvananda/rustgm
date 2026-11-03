//! Closed aggregate const values share typed canonical children, never source or byte images.
use super::{ConstId, ConstKind, Storage as Arguments};
#[cfg(test)]
use crate::support::retained::Allocation;
use crate::{
    support::{hash::HashMap, intern_slice::Slices, retained::Growth},
    types::{Kind, Store, TypeId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AggregateId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Contents {
    Fields { list: u32, variant: Option<u32> },
    Repeat { element: ConstId, length: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Node {
    ty: TypeId,
    contents: Contents,
}
#[derive(Default)]
pub(super) struct Storage {
    fields: Slices<ConstId>,
    nodes: Vec<Node>,
    index: HashMap<Node, AggregateId>,
}
#[cfg(test)]
impl Storage {
    pub(super) fn allocation(&self) -> Allocation {
        let mut result = self.fields.allocation();
        result.vector(&self.nodes);
        result.map(&self.index);
        result
    }
}
/// Nominal variant numbers are relative to their actual owning declaration.
#[derive(Clone, Copy)]
pub enum Aggregate<'a> {
    Fields {
        ty: TypeId,
        variant: Option<u32>,
        values: &'a [ConstId],
    },
    Repeat {
        ty: TypeId,
        element: ConstId,
        length: u64,
    },
}
impl Aggregate<'_> {
    #[must_use]
    pub const fn ty(self) -> TypeId {
        match self {
            Self::Fields { ty, .. } | Self::Repeat { ty, .. } => ty,
        }
    }
}
impl Store {
    pub(crate) fn intern_const_charged(
        &mut self,
        kind: ConstKind,
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ConstId, &'static str> {
        pay(1, 0)?;
        let arguments = self.arguments.as_deref();
        if let Some(id) = arguments.and_then(|a| a.constant_index.get(&kind)) {
            return Ok(*id);
        }
        let mut growth = Growth::vector::<ConstKind>(
            arguments.map_or(0, |a| a.constants.len()),
            arguments.map_or(0, |a| a.constants.capacity()),
            1,
        );
        growth.include(Growth::map::<ConstKind, ConstId>(
            arguments.map_or(0, |a| a.constant_index.len()),
            arguments.map_or(0, |a| a.constant_index.capacity()),
            1,
        ));
        if arguments.is_none() {
            growth.bytes = growth.bytes.saturating_add(size_of::<Arguments>());
        }
        pay(growth.work, growth.bytes)?;
        self.intern_const(kind)
    }
    #[must_use]
    pub fn aggregate_constant(&self, id: AggregateId) -> Option<Aggregate<'_>> {
        let storage = self.arguments.as_ref()?.aggregates.as_deref()?;
        let node = *storage.nodes.get(id.0 as usize)?;
        Some(match node.contents {
            Contents::Fields { list, variant } => Aggregate::Fields {
                ty: node.ty,
                variant,
                values: storage.fields.get(list)?,
            },
            Contents::Repeat { element, length } => Aggregate::Repeat {
                ty: node.ty,
                element,
                length,
            },
        })
    }
    /// Concrete canonical values are distinct from unresolved parameters.
    #[must_use]
    pub fn const_is_value(&self, id: ConstId) -> bool {
        matches!(
            self.const_kind(id),
            Some(ConstKind::Value(_) | ConstKind::Aggregate(_))
        )
    }
    /// Intern a typed product after semantic evaluation/nominal-field validation.
    /// This data producer grants no `ConstParamTy`, feature, layout or call permission.
    ///
    /// # Errors
    /// Rejects nonclosed types, invalid children, mismatched sequence types,
    /// resource payments and compact identity overflow.
    pub fn intern_aggregate_constant(
        &mut self,
        ty: TypeId,
        variant: Option<u32>,
        values: &[ConstId],
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ConstId, &'static str> {
        pay(values.len().saturating_add(1), 0)?;
        self.validate_const_fields(ty, variant, values)?;
        if let Some(Kind::Array { length, .. }) = self.kind(ty)
            && length != 0
            && let Some(&element) = values.first()
            && values.iter().all(|&value| value == element)
        {
            return self.intern_const_node(
                Node {
                    ty,
                    contents: Contents::Repeat { element, length },
                },
                &mut pay,
            );
        }
        let list = self.const_field_list(values, &mut pay)?;
        self.intern_const_node(
            Node {
                ty,
                contents: Contents::Fields { list, variant },
            },
            &mut pay,
        )
    }
    /// Canonical repeated arrays retain one child regardless of their length.
    ///
    /// # Errors
    /// Rejects a nonarray/invalid child, payment failure or compact overflow.
    pub fn intern_repeated_constant(
        &mut self,
        ty: TypeId,
        value: ConstId,
        mut pay: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ConstId, &'static str> {
        pay(1, 0)?;
        self.closed_const_type(ty)?;
        let Some(Kind::Array { element, length }) = self.kind(ty) else {
            return Err("canonical repeat requires a complete fixed array");
        };
        if !self.const_is_value(value) || self.const_type(value) != Some(element) {
            return Err("canonical repeat child differs from its element type");
        }
        if length == 0 {
            let list = self.const_field_list(&[], &mut pay)?;
            return self.intern_const_node(
                Node {
                    ty,
                    contents: Contents::Fields {
                        list,
                        variant: None,
                    },
                },
                &mut pay,
            );
        }
        self.intern_const_node(
            Node {
                ty,
                contents: Contents::Repeat {
                    element: value,
                    length,
                },
            },
            &mut pay,
        )
    }
    fn closed_const_type(&self, ty: TypeId) -> Result<(), &'static str> {
        if self.contains_parameters(ty) != Some(false)
            || self.contains_projections(ty) != Some(false)
            || self.needs_region_context(ty) != Some(false)
        {
            return Err("canonical aggregate const requires its closed completed type");
        }
        Ok(())
    }
    fn validate_const_fields(
        &self,
        ty: TypeId,
        variant: Option<u32>,
        values: &[ConstId],
    ) -> Result<(), &'static str> {
        self.closed_const_type(ty)?;
        if values.iter().any(|&value| !self.const_is_value(value)) {
            return Err("canonical aggregate const requires evaluated child values");
        }
        match self.kind(ty) {
            Some(Kind::Tuple(fields)) if variant.is_none() => {
                let expected = self
                    .list(fields)
                    .ok_or("canonical const tuple lost its field types")?;
                if expected.len() != values.len()
                    || expected
                        .iter()
                        .zip(values)
                        .any(|(&ty, &id)| self.const_type(id) != Some(ty))
                {
                    return Err("canonical const tuple fields differ from its type");
                }
            }
            Some(Kind::Array { element, length }) if variant.is_none() => {
                if length != values.len() as u64
                    || values
                        .iter()
                        .any(|&id| self.const_type(id) != Some(element))
                {
                    return Err("canonical const array fields differ from its type");
                }
            }
            Some(Kind::Adt(_, _)) => {}
            _ => return Err("canonical const product requires a tuple, array or nominal type"),
        }
        Ok(())
    }
    fn const_field_list(
        &mut self,
        values: &[ConstId],
        pay: &mut impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<u32, &'static str> {
        let prior = self
            .arguments
            .as_deref()
            .and_then(|a| a.aggregates.as_deref());
        let empty = Storage::default();
        let fields = &prior.unwrap_or(&empty).fields;
        if let Some(id) = fields.find_charged(values, |work| pay(work, 0))? {
            return Ok(id);
        }
        if fields.edge_count().saturating_add(values.len()) > 16_000_000
            || prior.is_some_and(|storage| storage.nodes.len() >= 1_000_000)
        {
            return Err("canonical aggregate const storage limit exceeded");
        }
        let mut growth = fields.interning_growth(values.len());
        self.const_storage_growth(&mut growth);
        pay(growth.work, growth.bytes)?;
        self.const_storage().fields.intern(values)
    }
    fn const_storage_growth(&self, growth: &mut Growth) {
        if self.arguments.is_none() {
            growth.bytes = growth.bytes.saturating_add(size_of::<Arguments>());
        }
        if self
            .arguments
            .as_deref()
            .is_none_or(|a| a.aggregates.is_none())
        {
            growth.bytes = growth.bytes.saturating_add(size_of::<Storage>());
        }
    }
    fn const_storage(&mut self) -> &mut Storage {
        self.arguments
            .get_or_insert_with(Default::default)
            .aggregates
            .get_or_insert_with(Default::default)
    }
    fn intern_const_node(
        &mut self,
        node: Node,
        pay: &mut impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ConstId, &'static str> {
        pay(1, 0)?;
        let prior = self
            .arguments
            .as_deref()
            .and_then(|a| a.aggregates.as_deref());
        if let Some(id) = prior.and_then(|storage| storage.index.get(&node)).copied() {
            return self.intern_const(ConstKind::Aggregate(id));
        }
        let empty = Storage::default();
        let storage = prior.unwrap_or(&empty);
        if storage.nodes.len() >= 1_000_000 || storage.fields.edge_count() > 16_000_000 {
            return Err("canonical aggregate const storage limit exceeded");
        }
        let id = AggregateId(
            u32::try_from(storage.nodes.len())
                .map_err(|_| "canonical aggregate const identity overflow")?,
        );
        let mut growth = Growth::vector::<Node>(storage.nodes.len(), storage.nodes.capacity(), 1);
        growth.include(Growth::map::<Node, AggregateId>(
            storage.index.len(),
            storage.index.capacity(),
            1,
        ));
        self.const_storage_growth(&mut growth);
        let arguments = self.arguments.as_deref();
        growth.include(Growth::vector::<ConstKind>(
            arguments.map_or(0, |a| a.constants.len()),
            arguments.map_or(0, |a| a.constants.capacity()),
            1,
        ));
        growth.include(Growth::map::<ConstKind, ConstId>(
            arguments.map_or(0, |a| a.constant_index.len()),
            arguments.map_or(0, |a| a.constant_index.capacity()),
            1,
        ));
        pay(growth.work, growth.bytes)?;
        let storage = self.const_storage();
        storage.nodes.push(node);
        storage.index.insert(node, id);
        self.intern_const(ConstKind::Aggregate(id))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/product_keys.rs"]
mod tests;
