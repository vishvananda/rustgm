//! Preserve the validated declaration identities and both lifetime scopes of a trait object.
use super::{Context, Declarations, DefId, Kind, Stage, g, s};
use crate::types::dynamic::ShapeId;

impl Stage {
    pub(super) fn dynamic_recipe(
        &mut self,
        facts: &Declarations,
        kind: Kind,
        outer: Context,
        inner: Context,
    ) -> Result<s::Type, &'static str> {
        let Kind::Dynamic {
            shape,
            arguments,
            region,
            ..
        } = kind
        else {
            return Err("source object recipe requires its actual checked dynamic type");
        };
        let actual = facts
            .types
            .dynamic_shape(shape)
            .ok_or("source trait object lost its checked shape")?;
        if !facts.types.valid_dynamic_arguments(shape, arguments) {
            return Err("source trait object changed its actual bound arguments");
        }
        let principal = actual
            .principal
            .map(|item| self.address(DefId(item.0)))
            .transpose()?;
        let arguments = self.arguments(facts, arguments, inner)?;
        let binder = self.binder(inner)?;
        let region = self.region(region, outer)?;
        let bindings = self.dynamic_bindings(facts, shape)?;
        let markers = self.dynamic_markers(facts, shape)?;
        self.reserve(4 * size_of::<s::DynamicType>())?;
        let objects = &mut self.cold()?.dynamics.objects;
        let reference = s::DynamicTypeRef(index(objects.len())?);
        objects.push(s::DynamicType {
            principal,
            principal_arguments: actual.principal_arguments,
            arguments,
            bindings,
            markers,
            binder,
            region,
        });
        Ok(s::Type::Dynamic(reference))
    }

    fn dynamic_bindings(
        &mut self,
        facts: &Declarations,
        shape: ShapeId,
    ) -> Result<g::Range, &'static str> {
        let bindings = facts
            .types
            .dynamic_bindings(shape)
            .ok_or("source trait object lost its associated bounds")?;
        self.reserve(
            bindings
                .len()
                .saturating_mul(4 * size_of::<s::DynamicBinding>()),
        )?;
        let start = index(self.rows.cold().dynamics.bindings.len())?;
        for binding in bindings {
            self.charge(1)?;
            let row = s::DynamicBinding {
                member: self.address(DefId(binding.item.0))?,
                owner: self.address(DefId(binding.trait_item.0))?,
                arguments: binding.arguments,
            };
            self.cold()?.dynamics.bindings.push(row);
        }
        Ok(g::Range {
            start,
            end: index(self.rows.cold().dynamics.bindings.len())?,
        })
    }

    fn dynamic_markers(
        &mut self,
        facts: &Declarations,
        shape: ShapeId,
    ) -> Result<g::Range, &'static str> {
        let markers = facts
            .types
            .dynamic_auto_traits(shape)
            .ok_or("source trait object lost its actual auto trait bounds")?;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("source trait object has no actual language catalog")?;
        self.reserve(
            markers
                .len()
                .saturating_mul(4 * size_of::<s::DynamicMarker>()),
        )?;
        let start = index(self.rows.cold().dynamics.markers.len())?;
        for &marker in markers {
            self.charge(1)?;
            let actual = catalog
                .auto_marker(marker.lang_item(), true)?
                .ok_or("source trait object lacks the actual auto trait declaration")?;
            let declaration = self.address(actual.definition)?;
            self.cold()?.dynamics.markers.push(s::DynamicMarker {
                marker,
                declaration,
            });
        }
        Ok(g::Range {
            start,
            end: index(self.rows.cold().dynamics.markers.len())?,
        })
    }
}

fn index(length: usize) -> Result<u32, &'static str> {
    u32::try_from(length).map_err(|_| "source trait object exceeded its row index range")
}
