//! Source Debug follows the pinned formatter helper recipe and ordinary field proof.
use super::{ItemKind, Kind, LexError, NodeId, Output, Span, error};
impl Output<'_> {
    pub(super) fn debug(&mut self, fields: &[NodeId]) -> Result<(), LexError> {
        use crate::syntax::item::Shape;
        let ItemKind::Struct { shape, .. } = self.site.item.kind else {
            return Err(error(
                self.site.original.start,
                "source Debug requires the struct profile",
            ));
        };
        (self.charge)(fields.len().saturating_add(1), 64)?;
        if fields.len() > 5 {
            return Err(error(
                self.site.original.start,
                "source Debug field-array helpers require a wider profile",
            ));
        }
        self.header("crate::fmt::Debug", false)?;
        self.text(
            "{ #[inline] fn fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result {",
        )?;
        let helper = if fields.is_empty() {
            "write_str".to_owned()
        } else {
            format!(
                "debug_{}_field{}_finish",
                if shape == Shape::Record {
                    "struct"
                } else {
                    "tuple"
                },
                fields.len()
            )
        };
        self.text("crate::fmt::Formatter::")?;
        self.text(&helper)?;
        self.text("(f,")?;
        self.label(
            self.site
                .item
                .name
                .ok_or_else(|| error(self.site.original.start, "derived Debug item has no name"))?,
        )?;
        for (index, &field) in fields.iter().enumerate() {
            let mut field = field;
            while let Kind::Attributed { value, .. } = self.site.ast.node(field).kind {
                field = value;
            }
            let Kind::FieldDefinition { name, .. } = self.site.ast.node(field).kind else {
                return Err(error(
                    self.site.original.start,
                    "source Debug lost its actual field",
                ));
            };
            self.text(",")?;
            if shape == Shape::Record {
                self.label(name.ok_or_else(|| {
                    error(self.site.original.start, "derived record field has no name")
                })?)?;
                self.text(",")?;
            }
            // rustc adds an extra reference to the final possibly-unsized field.
            self.text(if index + 1 == fields.len() {
                "&&self."
            } else {
                "&self."
            })?;
            if let Some(name) = name {
                self.spelling(name, false)?;
            } else {
                self.text(&index.to_string())?;
            }
        }
        self.text(") } }")
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
