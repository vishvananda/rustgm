//! Compact independent type facts; structural children combine dependency flags.
use super::RegionId;

#[derive(Clone, Copy)]
pub(super) enum Property {
    Sized,
    WellFormed,
    Regions,
    FreeRegions,
    LateCallables,
    Parameters,
    Callables,
}
impl Property {
    const fn mask(self) -> u8 {
        1 << (self as u8)
    }
}
const DEPENDENCIES: u8 = Property::Regions.mask()
    | Property::FreeRegions.mask()
    | Property::LateCallables.mask()
    | Property::Parameters.mask()
    | Property::Callables.mask();

#[derive(Clone, Copy)]
pub(super) struct Properties {
    pub escaping: u16,
    flags: u8,
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
        }
    }
}
