//! Default constructs real fields through their selected ordinary Default methods.
use super::{ItemKind, LexError, NodeId, Output, error};
use crate::syntax::item::Shape;
impl Output<'_> {
    pub(super) fn default_members(&mut self, fields: &[NodeId]) -> Result<(), LexError> {
        if !self.select(3) {
            return Ok(());
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
