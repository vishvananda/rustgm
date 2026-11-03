//! Compact independent type facts; structural children combine dependency flags.
use super::{Kind, Primitive, RegionId, Store};

#[derive(Clone, Copy)]
pub(super) enum Property {
    Sized,
    WellFormed,
    Regions,
    FreeRegions,
    LateCallables,
    Parameters,
    Callables,
    Projections,
    NonStaticFreeRegions,
    Opaques,
    Nominals,
    ProjectedSizes,
    AnonymousCallables,
    TrivialCopy,
}
impl Property {
    const fn mask(self) -> u16 {
        1 << (self as u16)
    }
}
const DEPENDENCIES: u16 = Property::Regions.mask()
    | Property::FreeRegions.mask()
    | Property::LateCallables.mask()
    | Property::Parameters.mask()
    | Property::Callables.mask()
    | Property::Projections.mask()
    | Property::NonStaticFreeRegions.mask()
    | Property::Opaques.mask()
    | Property::Nominals.mask()
    | Property::ProjectedSizes.mask()
    | Property::AnonymousCallables.mask();

#[derive(Clone, Copy)]
pub(super) struct Properties {
    pub escaping: u16,
    flags: u16,
}
impl Default for Properties {
    fn default() -> Self {
        Self {
            escaping: 0,
            flags: Property::Sized.mask() | Property::WellFormed.mask(),
        }
    }
}
impl Properties {
    pub fn is_concrete(self) -> bool {
        self.escaping == 0
            && !self.has(Property::Parameters)
            && !self.has(Property::Projections)
            && !self.has(Property::NonStaticFreeRegions)
    }
    pub fn has(self, property: Property) -> bool {
        self.flags & property.mask() != 0
    }
    pub fn set(&mut self, property: Property, value: bool) {
        if value {
            self.flags |= property.mask();
        } else {
            self.flags &= !property.mask();
        }
    }
    /// Combine child dependencies and well-formedness. Sizedness depends on
    /// the parent's constructor: a reference remains sized even if its child is not.
    pub fn include(&mut self, child: Self) {
        self.flags |= child.flags & DEPENDENCIES;
        self.set(
            Property::WellFormed,
            self.has(Property::WellFormed) && child.has(Property::WellFormed),
        );
        self.escaping = self.escaping.max(child.escaping);
    }
    pub fn region(&mut self, region: RegionId) {
        self.set(Property::Regions, true);
        if let Some((depth, _)) = region.bound_parts() {
            self.escaping = self.escaping.max(depth + 1);
        } else {
            self.set(Property::FreeRegions, true);
            if region != RegionId::STATIC {
                self.set(Property::NonStaticFreeRegions, true);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/concrete_inputs.rs"]
mod tests;

impl Store {
    /// A syntactic union-field shortcut, never source Copy or value validity evidence.
    pub(crate) fn trivial_copy(&self, ty: super::TypeId) -> Option<bool> {
        self.properties
            .get(ty.0 as usize)
            .map(|properties| properties.has(Property::TrivialCopy))
    }
    pub(super) fn trivial_copy_kind(&self, kind: Kind) -> Result<bool, &'static str> {
        Ok(match kind {
            Kind::Never
            | Kind::FunctionItem { .. }
            | Kind::Pointer { .. }
            | Kind::Reference { mutable: false, .. } => true,
            Kind::Primitive(primitive) => primitive != Primitive::Str,
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Pattern { base: element, .. } => self
                .trivial_copy(element)
                .ok_or("missing trivial-copy type child")?,
            Kind::Tuple(list) => {
                let fields = self.list(list).ok_or("missing trivial-copy tuple fields")?;
                fields.len() <= 3 && fields.iter().all(|&ty| self.trivial_copy(ty) == Some(true))
            }
            _ => false,
        })
    }
}
