//! Pinned formatter derives emit ordinary trait methods and actual field goals.
use super::{ItemKind, Kind, LexError, NodeId, Output, error};
impl Output<'_> {
    pub(super) fn comparison_members(&mut self, fields: &[NodeId]) -> Result<(), LexError> {
        if self.select(3) {
            if !fields.is_empty() || !matches!(self.site.item.kind, ItemKind::Struct { .. }) {
                return Err(error(
                    self.site.original.start,
                    "this source Default requires a fieldless struct",
                ));
            }
            self.header("crate::default::Default", false)?;
            self.text("{ #[inline] fn default() -> Self { Self {} } }")?;
        }
        if self.select(4) {
            self.header("crate::cmp::Eq", false)?;
            self.text(
                "{ #[inline] #[doc(hidden)] #[coverage(off)] fn assert_fields_are_eq(&self) {",
            )?;
            for &field in fields {
                let (_, ty) = self.actual_field(field)?;
                self.text("let _: crate::cmp::AssertParamIsEq<")?;
                self.field(ty)?;
                self.text(">;")?;
            }
            self.text("} }")?;
        }
        if self.select(5) {
            self.header("crate::marker::StructuralPartialEq", false)?;
            self.text("{}")?;
            self.header("crate::cmp::PartialEq", false)?;
            self.text("{ #[inline] fn eq(&self, other: &Self) -> bool {")?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                self.fieldless_enum(fields)?;
                self.discriminants()?;
                self.text("__self_discr == __arg1_discr")?;
            } else if fields.is_empty() {
                self.text("true")?;
            } else {
                for (index, &field) in fields.iter().enumerate() {
                    if index != 0 {
                        self.text("&&")?;
                    }
                    self.member("self", field, index)?;
                    self.text("==")?;
                    self.member("other", field, index)?;
                }
            }
            self.text("} }")?;
        }
        self.hash_ordering(fields)?;
        Ok(())
    }
    fn hash_ordering(&mut self, fields: &[NodeId]) -> Result<(), LexError> {
        if self.select(6) {
            self.fieldless_enum(fields)?;
            self.header("crate::hash::Hash", false)?;
            self.text("{ #[inline] fn hash<__H: crate::hash::Hasher>(&self, state: &mut __H) {")?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                self.text("let __self_discr = crate::intrinsics::discriminant_value(self); crate::hash::Hash::hash(&__self_discr,state);")?;
            }
            self.text("} }")?;
        }
        if self.select(7) {
            self.fieldless_enum(fields)?;
            self.header("crate::cmp::Ord", false)?;
            self.text("{ #[inline] fn cmp(&self, other: &Self) -> crate::cmp::Ordering {")?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                self.discriminants()?;
                self.text("crate::cmp::Ord::cmp(&__self_discr, &__arg1_discr)")?;
            } else {
                self.text("crate::cmp::Ordering::Equal")?;
            }
            self.text("} }")?;
        }
        if self.select(8) {
            self.fieldless_enum(fields)?;
            self.header("crate::cmp::PartialOrd", false)?;
            self.text("{ #[inline] fn partial_cmp(&self, other: &Self) -> crate::option::Option<crate::cmp::Ordering> { crate::option::Option::Some(")?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                self.text("crate::cmp::Ord::cmp(self,other)")?;
            } else {
                self.text("crate::cmp::Ordering::Equal")?;
            }
            self.text(") } }")?;
        }
        Ok(())
    }
    fn discriminants(&mut self) -> Result<(), LexError> {
        self.text("let __self_discr = crate::intrinsics::discriminant_value(self); let __arg1_discr = crate::intrinsics::discriminant_value(other);")
    }
    fn fieldless_enum(&self, fields: &[NodeId]) -> Result<(), LexError> {
        (self.charge)(1, 0)?;
        if !fields.is_empty() {
            return Err(error(
                self.site.original.start,
                "this source comparison/hash requires a fieldless struct or enum",
            ));
        }
        Ok(())
    }
    fn actual_field(&self, mut field: NodeId) -> Result<(Option<super::Span>, NodeId), LexError> {
        while let Kind::Attributed { value, .. } = self.site.ast.node(field).kind {
            (self.charge)(1, 0)?;
            field = value;
        }
        (self.charge)(1, 0)?;
        let Kind::FieldDefinition { name, ty, .. } = self.site.ast.node(field).kind else {
            return Err(error(
                self.site.original.start,
                "derived comparison lost its actual field",
            ));
        };
        Ok((name, ty))
    }
    fn member(&mut self, owner: &str, field: NodeId, index: usize) -> Result<(), LexError> {
        let (name, _) = self.actual_field(field)?;
        self.text(owner)?;
        self.text(".")?;
        if let Some(name) = name {
            self.spelling(name, false)
        } else {
            self.text(&index.to_string())
        }
    }
}
