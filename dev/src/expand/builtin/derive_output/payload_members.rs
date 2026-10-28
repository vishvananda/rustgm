//! Generated patterns borrow only the actual active variant fields.
use super::{LexError, NodeId, Output, active_members::VariantMembers, error};
use crate::syntax::item::Shape;
impl Output<'_> {
    pub(super) fn nominal_name(&mut self) -> Result<(), LexError> {
        self.spelling(
            self.site.item.name.ok_or_else(|| {
                error(
                    self.site.original.start,
                    "derived nominal lost its actual name",
                )
            })?,
            false,
        )
    }
    pub(super) fn struct_pattern(
        &mut self,
        shape: Shape,
        fields: &[NodeId],
    ) -> Result<(), LexError> {
        self.nominal_name()?;
        self.field_pattern(shape, fields, "__derive_field_")
    }
    pub(super) fn variant_path(&mut self, row: &VariantMembers) -> Result<(), LexError> {
        self.nominal_name()?;
        self.text("::")?;
        self.spelling(row.variant.name, false)
    }
    pub(super) fn variant_pattern(
        &mut self,
        row: &VariantMembers,
        fields: &[NodeId],
        prefix: &str,
    ) -> Result<(), LexError> {
        self.variant_path(row)?;
        self.field_pattern(row.variant.shape, fields, prefix)
    }
    fn field_pattern(
        &mut self,
        shape: Shape,
        fields: &[NodeId],
        prefix: &str,
    ) -> Result<(), LexError> {
        match shape {
            Shape::Unit => Ok(()),
            shape => {
                self.text(if shape == Shape::Record { "{" } else { "(" })?;
                for (index, &field) in fields.iter().enumerate() {
                    (self.charge)(1, 0)?;
                    if shape == Shape::Record {
                        let (name, _) = self.actual_field(field)?;
                        self.spelling(
                            name.ok_or_else(|| {
                                error(
                                    self.site.original.start,
                                    "derived record field lost its name",
                                )
                            })?,
                            false,
                        )?;
                        self.text(":")?;
                    }
                    self.binding(prefix, index)?;
                    self.text(",")?;
                }
                self.text(if shape == Shape::Record { "}" } else { ")" })
            }
        }
    }
    pub(super) fn binding(&mut self, prefix: &str, index: usize) -> Result<(), LexError> {
        use std::fmt::Write as _;
        let digits = index.checked_ilog10().unwrap_or(0) as usize + 1;
        let length = prefix.len().saturating_add(digits);
        (self.charge)(length, length.saturating_add(size_of::<String>()))?;
        let mut name = String::new();
        name.try_reserve_exact(length).map_err(|_| {
            error(
                self.site.original.start,
                "derived binding allocation failed",
            )
        })?;
        name.push_str(prefix);
        write!(&mut name, "{index}").map_err(|_| {
            error(
                self.site.original.start,
                "derived binding formatting failed",
            )
        })?;
        self.text(&name)
    }
    pub(super) fn pair_pattern(
        &mut self,
        row: &VariantMembers,
        fields: &[NodeId],
    ) -> Result<(), LexError> {
        self.text("(")?;
        self.variant_pattern(row, fields, "__derive_left_")?;
        self.text(",")?;
        self.variant_pattern(row, fields, "__derive_right_")?;
        self.text(")=>{")
    }
}
