//! Source Debug follows the pinned formatter helper recipe and ordinary field proof.
use super::{LexError, NodeId, Output, Span, active_members::VariantMembers, error};
use crate::syntax::item::{ItemKind, Shape};
#[derive(Clone, Copy)]
enum Access {
    Receiver,
    Pattern,
}
fn helper(shape: Shape, fields: usize) -> &'static str {
    const STRUCT: [&str; 5] = [
        "debug_struct_field1_finish",
        "debug_struct_field2_finish",
        "debug_struct_field3_finish",
        "debug_struct_field4_finish",
        "debug_struct_field5_finish",
    ];
    const TUPLE: [&str; 5] = [
        "debug_tuple_field1_finish",
        "debug_tuple_field2_finish",
        "debug_tuple_field3_finish",
        "debug_tuple_field4_finish",
        "debug_tuple_field5_finish",
    ];
    match fields {
        0 => "write_str",
        count if shape == Shape::Record => STRUCT
            .get(count - 1)
            .copied()
            .unwrap_or("debug_struct_fields_finish"),
        count => TUPLE
            .get(count - 1)
            .copied()
            .unwrap_or("debug_tuple_fields_finish"),
    }
}
impl Output<'_> {
    pub(super) fn debug(
        &mut self,
        fields: &[NodeId],
        variants: &[VariantMembers],
    ) -> Result<(), LexError> {
        (self.charge)(fields.len().saturating_add(1), 64)?;
        self.header("crate::fmt::Debug", false)?;
        self.text(
            "{ #[inline] fn fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result {",
        )?;
        if let ItemKind::Struct { shape, .. } = self.site.item.kind {
            let name =
                self.site.item.name.ok_or_else(|| {
                    error(self.site.original.start, "derived Debug item has no name")
                })?;
            self.debug_call(shape, name, fields, Access::Receiver)?;
        } else {
            self.text("match self{")?;
            for row in variants {
                let fields = &fields[row.fields.clone()];
                self.variant_pattern(row, fields, "__derive_field_")?;
                self.text("=>{")?;
                self.debug_call(row.variant.shape, row.variant.name, fields, Access::Pattern)?;
                self.text("},")?;
            }
            self.text("}")?;
        }
        self.text("} }")
    }
    fn debug_call(
        &mut self,
        shape: Shape,
        name: Span,
        fields: &[NodeId],
        access: Access,
    ) -> Result<(), LexError> {
        let arrays = fields.len() > 5;
        if arrays {
            self.debug_arrays(shape, fields, access)?;
        }
        self.text("crate::fmt::Formatter::")?;
        self.text(helper(shape, fields.len()))?;
        self.text("(f,")?;
        self.label(name)?;
        if arrays {
            if shape == Shape::Record {
                self.text(",names")?;
            }
            self.text(",values")?;
        } else {
            for (index, &field) in fields.iter().enumerate() {
                self.text(",")?;
                if shape == Shape::Record {
                    self.debug_label(field)?;
                    self.text(",")?;
                }
                self.debug_value(access, index, field, fields.len())?;
            }
        }
        self.text(")")
    }
    fn debug_arrays(
        &mut self,
        shape: Shape,
        fields: &[NodeId],
        access: Access,
    ) -> Result<(), LexError> {
        if shape == Shape::Record {
            self.text("let names: &'static _ = &[")?;
            for &field in fields {
                self.debug_label(field)?;
                self.text(",")?;
            }
            self.text("];")?;
        }
        self.text("let values: &[&dyn crate::fmt::Debug] = &[")?;
        for (index, &field) in fields.iter().enumerate() {
            self.debug_value(access, index, field, fields.len())?;
            self.text(",")?;
        }
        self.text("];")
    }
    fn debug_label(&mut self, field: NodeId) -> Result<(), LexError> {
        let (name, _) = self.actual_field(field)?;
        self.label(
            name.ok_or_else(|| {
                error(self.site.original.start, "derived record field has no name")
            })?,
        )
    }
    fn debug_value(
        &mut self,
        access: Access,
        index: usize,
        field: NodeId,
        fields: usize,
    ) -> Result<(), LexError> {
        // The last field needs an extra indirection for possibly-unsized types.
        match access {
            Access::Receiver => {
                self.text(if index + 1 == fields {
                    "&&self."
                } else {
                    "&self."
                })?;
                if let Some(name) = self.actual_field(field)?.0 {
                    self.spelling(name, false)?;
                } else {
                    self.binding("", index)?;
                }
                Ok(())
            }
            Access::Pattern => {
                if index + 1 == fields {
                    self.text("&")?;
                }
                self.binding("__derive_field_", index)
            }
        }
    }
    fn label(&mut self, span: Span) -> Result<(), LexError> {
        use unicode_normalization::UnicodeNormalization;
        let spelling = self
            .site
            .source
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| error(span.start, "derived label is outside its source"))?;
        (self.charge)(
            spelling.len().saturating_add(1),
            spelling.len().saturating_mul(8).saturating_add(16),
        )?;
        let name: String = spelling
            .strip_prefix("r#")
            .unwrap_or(spelling)
            .nfc()
            .collect();
        let mut quoted = String::new();
        quoted
            .try_reserve(name.len().saturating_mul(4).saturating_add(2))
            .map_err(|_| error(span.start, "derived label allocation failed"))?;
        quoted.push('"');
        for character in name.chars() {
            quoted.extend(character.escape_default());
        }
        quoted.push('"');
        self.text(&quoted)
    }
}
