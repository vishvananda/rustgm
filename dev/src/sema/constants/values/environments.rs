//! Closure storage keeps its canonical capture tuple and completed physical owner.
use super::{Reader, StoredShape};
use crate::{
    abi::layout::{Layout, Shape},
    types::{Kind, TypeId},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/environment_shapes.rs"]
mod tests;

impl<'a> Reader<'a> {
    pub(super) fn stored_closure(
        self,
        environment: TypeId,
        layout: &Layout,
    ) -> Result<StoredShape<'a>, &'static str> {
        let Some(Kind::Tuple(fields)) = self.facts.types.kind(environment) else {
            return Err("stored closure requires its canonical capture tuple");
        };
        let retained = self
            .facts
            .layouts
            .get(environment)
            .ok_or("stored closure requires completed environment layout")?;
        let (
            Shape::Aggregate {
                variants: actual,
                tag: None,
            },
            Shape::Aggregate {
                variants: expected,
                tag: None,
            },
        ) = (layout.shape, retained.shape)
        else {
            return Err("stored closure requires its retained environment product");
        };
        if (
            layout.size,
            layout.align,
            layout.inhabited,
            layout.contains_reference,
            layout.contains_explicit_align,
            actual.start,
            actual.end,
        ) != (
            retained.size,
            retained.align,
            retained.inhabited,
            retained.contains_reference,
            retained.contains_explicit_align,
            expected.start,
            expected.end,
        ) {
            return Err("stored closure differs from its exact environment layout");
        }
        let [variant] = self.facts.layouts.variants(layout) else {
            return Err("stored closure requires one capture product");
        };
        if variant.fields != fields {
            return Err("stored closure layout has different capture types");
        }
        self.stored_product(layout, None, false)
    }
}
