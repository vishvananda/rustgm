//! Type-generic Clone uses real field calls; paired Copy never changes its bound.
use super::{
    GenericFlavor, ItemKind, Kind, LexError, NodeId, Output, active_members::VariantMembers, error,
};
use crate::syntax::item::Shape;
impl Output<'_> {
    pub(super) fn clone_members(
        &mut self,
        fields: &[NodeId],
        variants: &[VariantMembers],
        copy: bool,
    ) -> Result<(), LexError> {
        self.select(1);
        let mut type_parameters = false;
        for &node in self.site.ast.list(self.site.item.generics) {
            (self.charge)(1, 0)?;
            let Kind::Generic(index) = self.site.ast.node(node).kind else {
                return Err(error(
                    self.site.original.start,
                    "derived Clone lost its generic parameter",
                ));
            };
            type_parameters |= self.site.ast.generics[index as usize].flavor == GenericFlavor::Type;
        }
        if copy && !type_parameters {
            self.header("crate::clone::TrivialClone", true)?;
            self.text("{}")?;
        }
        self.header("crate::clone::Clone", false)?;
        self.text("{ #[inline] fn clone(&self)->Self{")?;
        if copy && !type_parameters {
            for &field in fields {
                let (_, ty) = self.actual_field(field)?;
                self.text("let _:crate::clone::AssertParamIsClone<")?;
                self.field(ty)?;
                self.text(">;")?;
            }
            self.text("*self")?;
        } else if let ItemKind::Struct { shape, .. } = self.site.item.kind {
            self.text("match self{")?;
            self.struct_pattern(shape, fields)?;
            self.text("=>")?;
            self.nominal_name()?;
            self.clone_fields(shape, fields, true)?;
            self.text("}")?;
        } else {
            self.text("match self{")?;
            for row in variants {
                let fields = &fields[row.fields.clone()];
                self.variant_pattern(row, fields, "__derive_field_")?;
                self.text("=>")?;
                self.variant_path(row)?;
                self.clone_fields(row.variant.shape, fields, true)?;
                self.text(",")?;
            }
            self.text("}")?;
        }
        self.text("} }")
    }
    fn clone_fields(
        &mut self,
        shape: Shape,
        fields: &[NodeId],
        bindings: bool,
    ) -> Result<(), LexError> {
        if shape == Shape::Unit {
            return Ok(());
        }
        self.text(if shape == Shape::Record { "{" } else { "(" })?;
        for (index, &field) in fields.iter().enumerate() {
            (self.charge)(1, 0)?;
            if shape == Shape::Record {
                let (name, _) = self.actual_field(field)?;
                self.spelling(
                    name.ok_or_else(|| {
                        error(
                            self.site.original.start,
                            "derived Clone record field lost its name",
                        )
                    })?,
                    false,
                )?;
                self.text(":")?;
            }
            self.text("crate::clone::Clone::clone(")?;
            if bindings {
                self.binding("__derive_field_", index)?;
            } else {
                self.text("&")?;
                self.member("self", field, index)?;
            }
            self.text("),")?;
        }
        self.text(if shape == Shape::Record { "}" } else { ")" })
    }
}
