//! Object bound shapes separate declaration identities from substitutable inputs.
mod guarantees;
mod promises;
use super::{GenericArgs, ItemId, Store, arguments::Argument, auto_traits::AutoTrait};
use crate::support::hash::HashMap;
use crate::support::intern_slice::Slices;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeId(pub u32);

/// An equality supplies this member's complete parent application and result.
/// Dyn-compatible associated types have no own generic arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Binding {
    pub item: ItemId,
    pub trait_item: ItemId,
    pub arguments: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shape {
    pub principal: Option<ItemId>,
    pub principal_arguments: u32,
    bindings: u32,
    autos: u32,
    arity: u32,
}

/// Principal arguments precede each associated equality's parent arguments and
/// result. All these inputs belong to the object's trait binder. The object's
/// lifetime is outside that binder and is not an argument of this shape.
#[derive(Clone, Copy)]
pub struct Bounds<'a> {
    pub principal: Option<ItemId>,
    pub principal_arguments: u32,
    pub bindings: &'a [Binding],
    pub auto_traits: &'a [AutoTrait],
}

#[derive(Default)]
pub(super) struct Storage {
    shapes: Vec<Shape>,
    index: HashMap<Shape, ShapeId>,
    bindings: Slices<Binding>,
    autos: Slices<AutoTrait>,
    inherited_autos: HashMap<ItemId, u8>,
    outlives: HashMap<ItemId, Box<[guarantees::Promise]>>,
}

impl Store {
    #[must_use]
    pub fn has_dynamic_shapes(&self) -> bool {
        self.dynamics.is_some()
    }

    /// Read the principal application without opening the object's trait binder.
    /// # Errors
    /// Rejects a non-object, malformed bound shape or invalid mixed input layout.
    pub fn dynamic_principal(
        &mut self,
        ty: super::TypeId,
    ) -> Result<Option<super::traits::ApplicationId>, &'static str> {
        let Some(super::Kind::Dynamic {
            shape, arguments, ..
        }) = self.kind(ty)
        else {
            return Err("principal lookup requires an object type");
        };
        let shape = self
            .dynamic_shape(shape)
            .ok_or("invalid object bound shape")?;
        let Some(item) = shape.principal else {
            return Ok(None);
        };
        let arguments = self.arguments(arguments).ok_or("invalid object inputs")?;
        if arguments.len() < shape.principal_arguments as usize {
            return Err("invalid object principal arity");
        }
        let arguments = arguments
            .take(shape.principal_arguments as usize)
            .collect::<Vec<_>>();
        let arguments = self.intern_arguments(&arguments)?;
        self.intern_trait_application(super::traits::Application { item, arguments })
            .map(Some)
    }

    pub(super) fn dynamic_properties(
        &self,
        kind: super::Kind,
    ) -> Result<super::Properties, &'static str> {
        use super::Property;
        let super::Kind::Dynamic {
            arguments,
            bound_regions,
            region,
            ..
        } = kind
        else {
            return Err("object properties require a dynamic type");
        };
        let mut result = super::Properties::default();
        self.argument_properties(arguments, &mut result)?;
        result.escaping = result.escaping.saturating_sub(1);
        result.region(region);
        result.set(Property::Sized, false);
        result.set(Property::Callables, true);
        result.set(
            Property::LateCallables,
            result.has(Property::LateCallables) || bound_regions != 0,
        );
        Ok(result)
    }
    /// Canonicalize bound declaration identities without deciding dyn compatibility.
    /// Semantic formation validates trait kinds, parameter kinds and equalities.
    /// # Errors
    /// Rejects empty bounds, unordered/repeated associated members and arena overflow.
    pub fn intern_dynamic_shape(&mut self, bounds: Bounds<'_>) -> Result<ShapeId, &'static str> {
        let arity = bounds.arity()?;
        let mut autos = bounds.auto_traits.to_vec();
        autos.sort_unstable();
        autos.dedup();
        let storage = self.dynamics.get_or_insert_with(Default::default);
        let shape = Shape {
            principal: bounds.principal,
            principal_arguments: bounds.principal_arguments,
            bindings: storage.bindings.intern(bounds.bindings)?,
            autos: storage.autos.intern(&autos)?,
            arity,
        };
        if let Some(&id) = storage.index.get(&shape) {
            return Ok(id);
        }
        let id = ShapeId(
            u32::try_from(storage.shapes.len()).map_err(|_| "too many dynamic bound shapes")?,
        );
        storage.shapes.push(shape);
        storage.index.insert(shape, id);
        Ok(id)
    }

    #[must_use]
    pub fn dynamic_shape(&self, id: ShapeId) -> Option<Shape> {
        self.dynamics.as_ref()?.shapes.get(id.0 as usize).copied()
    }
    #[must_use]
    pub fn dynamic_bindings(&self, id: ShapeId) -> Option<&[Binding]> {
        let shape = self.dynamic_shape(id)?;
        self.dynamics.as_ref()?.bindings.get(shape.bindings)
    }
    #[must_use]
    pub fn dynamic_auto_traits(&self, id: ShapeId) -> Option<&[AutoTrait]> {
        let shape = self.dynamic_shape(id)?;
        self.dynamics.as_ref()?.autos.get(shape.autos)
    }

    /// Freeze the principal's unconditional Self marker promises once.
    /// # Errors
    /// Rejects conflicting declarations after a promise has been published.
    pub(crate) fn declare_dynamic_autos(
        &mut self,
        item: ItemId,
        autos: u8,
    ) -> Result<(), &'static str> {
        let storage = self.dynamics.get_or_insert_with(Default::default);
        if storage
            .inherited_autos
            .get(&item)
            .is_some_and(|previous| *previous != autos)
        {
            return Err("object auto-trait promises changed after publication");
        }
        storage.inherited_autos.insert(item, autos);
        Ok(())
    }

    /// Object marker permissions come from its explicit or inherited bounds.
    pub(crate) fn dynamic_has_auto(&self, id: ShapeId, marker: AutoTrait) -> bool {
        self.dynamic_auto_traits(id)
            .is_some_and(|autos| autos.contains(&marker))
            || self
                .dynamic_shape(id)
                .and_then(|shape| shape.principal)
                .and_then(|principal| self.dynamics.as_ref()?.inherited_autos.get(&principal))
                .is_some_and(|autos| autos & (1 << marker as u8) != 0)
    }

    /// Shape validation is distinct from proving that these bounds form an object.
    #[must_use]
    pub fn valid_dynamic_arguments(&self, id: ShapeId, args: GenericArgs) -> bool {
        let Some(shape) = self.dynamic_shape(id) else {
            return false;
        };
        let Some(bindings) = self.dynamic_bindings(id) else {
            return false;
        };
        let Some(mut args) = self.arguments(args) else {
            return false;
        };
        if args.len() != shape.arity as usize {
            return false;
        }
        for _ in 0..shape.principal_arguments {
            args.next();
        }
        bindings.iter().all(|binding| {
            matches!(
                args.nth(binding.arguments as usize),
                Some(Argument::Type(_))
            )
        })
    }
}

impl Bounds<'_> {
    fn arity(&self) -> Result<u32, &'static str> {
        if self.principal.is_none() {
            if self.auto_traits.is_empty() {
                return Err("dynamic bounds require at least one trait");
            }
            if self.principal_arguments != 0 || !self.bindings.is_empty() {
                return Err("associated object bounds require a principal trait");
            }
        }
        if self
            .bindings
            .windows(2)
            .any(|pair| pair[0].item.0 >= pair[1].item.0)
        {
            return Err("dynamic associated members must be unique and ordered");
        }
        self.bindings
            .iter()
            .try_fold(self.principal_arguments, |arity, binding| {
                arity
                    .checked_add(binding.arguments)
                    .and_then(|count| count.checked_add(1))
                    .ok_or("too many dynamic bound arguments")
            })
    }
}

/// Replace a trait binder's inputs while leaving the enclosing object lifetime alone.
pub(crate) fn open(
    types: &mut Store,
    ty: super::TypeId,
    replacement: &mut super::substitute::Bound,
) -> Result<super::TypeId, &'static str> {
    let Some(super::Kind::Dynamic {
        shape,
        arguments,
        region,
        ..
    }) = types.kind(ty)
    else {
        return Err("object binder substitution requires a dynamic type");
    };
    let mut arguments = types
        .arguments(arguments)
        .ok_or("invalid object binder inputs")?
        .collect::<Vec<_>>();
    for argument in &mut arguments {
        *argument = replacement.apply_argument(types, *argument)?;
    }
    let arguments = types.intern_arguments(&arguments)?;
    types.intern(super::Kind::Dynamic {
        shape,
        arguments,
        region,
        bound_regions: 0,
    })
}
