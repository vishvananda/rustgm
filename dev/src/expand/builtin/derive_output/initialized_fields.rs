//! Default constructs real fields through their selected ordinary Default methods.
use super::{ItemKind, Kind, LexError, NodeId, Output, error};
use crate::syntax::item::Shape;
impl Output<'_> {
    pub(super) fn default_members(&mut self, fields: &[NodeId]) -> Result<(), LexError> {
        if !self.select(3) {
            return Ok(());
        }
        if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
            let mut node = self.site.enum_default.ok_or_else(|| {
                error(
                    self.site.original.start,
                    "enum Default lost its actual selected variant",
                )
            })?;
            while let Kind::Attributed { value, .. } = self.site.ast.node(node).kind {
                node = value;
            }
            let Kind::Variant(index) = self.site.ast.node(node).kind else {
                return Err(error(
                    self.site.original.start,
                    "enum Default selected another owner",
                ));
            };
            let variant = self.site.ast.variants[index as usize];
            if variant.shape != Shape::Unit {
                return Err(error(
                    variant.name.start,
                    "enum Default requires a unit variant",
                ));
            }
            self.header("crate::default::Default", false)?;
            self.text("{ #[inline] fn default()->Self{Self::")?;
            self.spelling(variant.name, false)?;
            return self.text("} }");
        }
        let ItemKind::Struct { shape, .. } = self.site.item.kind else {
            return Err(error(
                self.site.original.start,
                "source Default requires its supported struct owner",
            ));
        };
        self.header("crate::default::Default", false)?;
        self.text("{ #[inline] fn default()->Self{Self")?;
        self.text(if shape == Shape::Tuple { "(" } else { "{" })?;
        for &field in fields {
            (self.charge)(1, 0)?;
            let (name, _) = self.actual_field(field)?;
            if shape == Shape::Record {
                self.spelling(
                    name.ok_or_else(|| {
                        error(
                            self.site.original.start,
                            "Default record field lost its name",
                        )
                    })?,
                    false,
                )?;
                self.text(":")?;
            } else if name.is_some() || shape == Shape::Unit {
                return Err(error(
                    self.site.original.start,
                    "Default field has a different actual constructor shape",
                ));
            }
            self.text("crate::default::Default::default(),")?;
        }
        self.text(if shape == Shape::Tuple {
            ")} }"
        } else {
            "}} }"
        })
    }
}
