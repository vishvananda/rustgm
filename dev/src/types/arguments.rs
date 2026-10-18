//! Canonical generic arguments in declaration order, with a lifetime-only fast path.
use super::{GenericArgs, Kind, Primitive, RegionId, Store, TypeId};
use crate::support::intern_slice::Slices;
use std::collections::HashMap;

const MIXED: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstId(pub u32);
/// Evaluated target-width bits, including two's-complement signed integers.
/// Source literal interpretation and const evaluation precede canonicalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Constant {
    pub ty: TypeId,
    pub bits: u128,
}
/// An evaluated value or a rigid const parameter, never an inference variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConstKind {
    Value(Constant),
    Parameter(super::parameters::ParameterId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Argument {
    Lifetime(RegionId),
    Type(TypeId),
    Const(ConstId),
}
/// Borrowed traversal of either storage representation; never allocates.
pub enum Arguments<'a> {
    Lifetimes(std::slice::Iter<'a, RegionId>),
    Mixed(std::slice::Iter<'a, Argument>),
}
impl Iterator for Arguments<'_> {
    type Item = Argument;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Lifetimes(values) => values.next().copied().map(Argument::Lifetime),
            Self::Mixed(values) => values.next().copied(),
        }
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::Lifetimes(values) => values.nth(n).copied().map(Argument::Lifetime),
            Self::Mixed(values) => values.nth(n).copied(),
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}
impl ExactSizeIterator for Arguments<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Lifetimes(values) => values.len(),
            Self::Mixed(values) => values.len(),
        }
    }
}
impl std::iter::FusedIterator for Arguments<'_> {}

#[derive(Default)]
pub(super) struct Storage {
    pub(super) parameters: super::parameters::Storage,
    pub(super) traits: Option<Box<super::traits::Storage>>,
    pub(super) projections: Option<Box<super::projection::Storage>>,
    mixed: Slices<Argument>,
    constants: Vec<ConstKind>,
    constant_index: HashMap<ConstKind, ConstId>,
}
impl Store {
    pub(crate) fn find_regions(&self, values: &[RegionId]) -> Option<GenericArgs> {
        self.regions.find(values).map(GenericArgs)
    }
    /// The caller preserves a mixed list's argument kinds while mapping its values.
    pub(crate) fn find_mixed_arguments(&self, values: &[Argument]) -> Option<GenericArgs> {
        self.arguments
            .as_ref()?
            .mixed
            .find(values)
            .map(|id| GenericArgs(id | MIXED))
    }
    /// Lifetime-only consumers must reject a mixed list rather than erase arguments.
    #[must_use]
    pub fn region_args(&self, id: GenericArgs) -> Option<&[RegionId]> {
        if id.0 & MIXED != 0 {
            return None;
        }
        self.regions.get(id.0)
    }
    #[must_use]
    pub fn arguments(&self, id: GenericArgs) -> Option<Arguments<'_>> {
        if id.0 & MIXED == 0 {
            self.regions
                .get(id.0)
                .map(|args| Arguments::Lifetimes(args.iter()))
        } else if id.0 != MIXED {
            self.arguments
                .as_ref()?
                .mixed
                .get(id.0 & !MIXED)
                .map(|args| Arguments::Mixed(args.iter()))
        } else {
            None
        }
    }
    /// # Errors
    /// Rejects compact argument arena overflow.
    pub fn intern_regions(&mut self, values: &[RegionId]) -> Result<GenericArgs, &'static str> {
        let id = self.regions.intern(values)?;
        if id >= MIXED {
            return Err("too many canonical lifetime argument lists");
        }
        Ok(GenericArgs(id))
    }
    /// Intern all arguments, preserving their kinds and declaration order.
    /// An all-lifetime list has the same identity as `intern_regions`.
    ///
    /// # Errors
    /// Rejects invalid type/const children and compact argument arena overflow.
    pub fn intern_arguments(&mut self, values: &[Argument]) -> Result<GenericArgs, &'static str> {
        if values
            .iter()
            .all(|arg| matches!(arg, Argument::Lifetime(_)))
        {
            let regions: Vec<_> = values
                .iter()
                .filter_map(|arg| match arg {
                    Argument::Lifetime(region) => Some(*region),
                    _ => None,
                })
                .collect();
            return self.intern_regions(&regions);
        }
        for value in values {
            let valid = match value {
                Argument::Lifetime(_) => true,
                Argument::Type(ty) => self.kind(*ty).is_some(),
                Argument::Const(id) => self.const_kind(*id).is_some(),
            };
            if !valid {
                return Err("invalid canonical generic argument");
            }
        }
        let id = self
            .arguments
            .get_or_insert_with(Default::default)
            .mixed
            .intern(values)?;
        if id >= MIXED {
            return Err("too many canonical generic argument lists");
        }
        Ok(GenericArgs(id | MIXED))
    }
    #[must_use]
    pub fn constant(&self, id: ConstId) -> Option<Constant> {
        match self.const_kind(id)? {
            ConstKind::Value(value) => Some(value),
            ConstKind::Parameter(_) => None,
        }
    }
    #[must_use]
    pub fn const_kind(&self, id: ConstId) -> Option<ConstKind> {
        self.arguments
            .as_ref()?
            .constants
            .get(id.0 as usize)
            .copied()
    }
    #[must_use]
    pub fn const_type(&self, id: ConstId) -> Option<TypeId> {
        match self.const_kind(id)? {
            ConstKind::Value(value) => Some(value.ty),
            ConstKind::Parameter(id) => match self.parameter(id)?.kind {
                super::parameters::ParameterKind::Const { ty } => Some(ty),
                super::parameters::ParameterKind::Type { .. } => None,
            },
        }
    }
    /// # Errors
    /// Rejects non-const-parameter types, invalid scalar bits and compact overflow.
    pub fn intern_constant(&mut self, value: Constant) -> Result<ConstId, &'static str> {
        self.intern_const(ConstKind::Value(value))
    }
    /// # Errors
    /// Rejects invalid scalar values, non-const parameter identities and compact overflow.
    pub fn intern_const(&mut self, kind: ConstKind) -> Result<ConstId, &'static str> {
        let valid = match kind {
            ConstKind::Value(value) => self.valid_constant(value),
            ConstKind::Parameter(id) => self.parameter(id).is_some_and(|parameter| {
                matches!(
                    parameter.kind,
                    super::parameters::ParameterKind::Const { .. }
                )
            }),
        };
        if !valid {
            return Err("invalid const generic scalar type or bits");
        }
        let storage = self.arguments.get_or_insert_with(Default::default);
        if let Some(id) = storage.constant_index.get(&kind) {
            return Ok(*id);
        }
        let id = ConstId(
            u32::try_from(storage.constants.len())
                .map_err(|_| "too many canonical generic constants")?,
        );
        storage.constants.push(kind);
        storage.constant_index.insert(kind, id);
        Ok(id)
    }
    fn valid_constant(&self, value: Constant) -> bool {
        match self.kind(value.ty) {
            Some(Kind::Primitive(Primitive::Bool)) => value.bits <= 1,
            Some(Kind::Primitive(Primitive::Char)) => u32::try_from(value.bits)
                .ok()
                .and_then(char::from_u32)
                .is_some(),
            Some(Kind::Primitive(primitive)) => primitive
                .integer()
                .is_some_and(|(_, width)| width == 128 || value.bits < (1_u128 << width)),
            _ => false,
        }
    }
    /// Whether complete arguments contain no caller parameters, external regions
    /// or unresolved projections. This structural fact is not implementation evidence.
    /// # Errors
    /// Rejects unknown argument lists or type identities.
    pub fn arguments_are_concrete(&self, args: GenericArgs) -> Result<bool, &'static str> {
        let mut properties = super::Properties::default();
        self.argument_properties(args, &mut properties)?;
        Ok(properties.is_concrete())
    }
    pub(super) fn argument_properties(
        &self,
        args: GenericArgs,
        result: &mut super::Properties,
    ) -> Result<(), &'static str> {
        for argument in self.arguments(args).ok_or("invalid generic arguments")? {
            match argument {
                Argument::Lifetime(region) => {
                    result.region(region);
                }
                Argument::Type(ty) => {
                    let child = self
                        .properties
                        .get(ty.0 as usize)
                        .ok_or("invalid generic type argument")?;
                    result.include(*child);
                }
                Argument::Const(id) => {
                    if matches!(self.const_kind(id), Some(ConstKind::Parameter(_))) {
                        result.set(super::Property::Parameters, true);
                    }
                }
            }
        }
        Ok(())
    }
}
