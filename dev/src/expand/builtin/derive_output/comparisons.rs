//! Derived comparison and hashing bodies retain real field and discriminant calls.
use super::{ItemKind, Kind, LexError, NodeId, Output, active_members::VariantMembers, error};
impl Output<'_> {
    pub(super) fn comparison_members(
        &mut self,
        fields: &[NodeId],
        variants: &[VariantMembers],
    ) -> Result<(), LexError> {
        self.default_members(fields)?;
        if self.select(4) {
            self.header("crate::cmp::Eq", false)?;
            self.text(
                "{ #[inline] #[doc(hidden)] #[coverage(off)] fn assert_fields_are_eq(&self){",
            )?;
            for &field in fields {
                let (_, ty) = self.actual_field(field)?;
                self.text("let _:crate::cmp::AssertParamIsEq<")?;
                self.field(ty)?;
                self.text(">;")?;
            }
            self.text("} }")?;
        }
        if self.select(5) {
            self.header("crate::marker::StructuralPartialEq", false)?;
            self.text("{}")?;
            self.header("crate::cmp::PartialEq", false)?;
            self.text("{ #[inline] fn eq(&self,other:&Self)->bool{")?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                self.text("match (self,other){")?;
                for row in variants {
                    let selected = &fields[row.fields.clone()];
                    self.pair_pattern(row, selected)?;
                    self.equal_fields(selected, true)?;
                    self.text("},")?;
                }
                self.text("_=>false}")?;
            } else {
                self.equal_fields(fields, false)?;
            }
            self.text("} }")?;
        }
        self.hash_ordering(fields, variants)
    }
    fn equal_fields(&mut self, fields: &[NodeId], bindings: bool) -> Result<(), LexError> {
        if fields.is_empty() {
            return self.text("true");
        }
        for (index, &field) in fields.iter().enumerate() {
            (self.charge)(1, 0)?;
            if index != 0 {
                self.text("&&")?;
            }
            if bindings {
                self.binding("__derive_left_", index)?;
            } else {
                self.member("self", field, index)?;
            }
            self.text("==")?;
            if bindings {
                self.binding("__derive_right_", index)?;
            } else {
                self.member("other", field, index)?;
            }
        }
        Ok(())
    }
    fn hash_ordering(
        &mut self,
        fields: &[NodeId],
        variants: &[VariantMembers],
    ) -> Result<(), LexError> {
        if self.select(6) {
            self.header("crate::hash::Hash", false)?;
            let hasher = self.hasher_name()?;
            self.text("{ #[inline] fn hash<")?;
            self.text(&hasher)?;
            self.text(":crate::hash::Hasher>(&self,state:&mut ")?;
            self.text(&hasher)?;
            self.text("){")?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                if variants.len() > 1 {
                    self.text("let __self_discr=crate::intrinsics::discriminant_value(self);crate::hash::Hash::hash(&__self_discr,state);")?;
                }
                self.text("match self{")?;
                for row in variants {
                    let selected = &fields[row.fields.clone()];
                    self.variant_pattern(row, selected, "__derive_field_")?;
                    self.text("=>{")?;
                    self.hash_fields(selected, true)?;
                    self.text("},")?;
                }
                self.text("}")?;
            } else {
                self.hash_fields(fields, false)?;
            }
            self.text("} }")?;
        }
        for (slot, partial) in [(7, false), (8, true)] {
            if !self.select(slot) {
                continue;
            }
            self.header(
                if partial {
                    "crate::cmp::PartialOrd"
                } else {
                    "crate::cmp::Ord"
                },
                false,
            )?;
            self.text(if partial{"{ #[inline] fn partial_cmp(&self,other:&Self)->crate::option::Option<crate::cmp::Ordering>{"}else{"{ #[inline] fn cmp(&self,other:&Self)->crate::cmp::Ordering{"})?;
            if matches!(self.site.item.kind, ItemKind::Enum { .. }) {
                if variants.len() > 1 {
                    self.text("let __self_discr=crate::intrinsics::discriminant_value(self);let __arg1_discr=crate::intrinsics::discriminant_value(other);")?;
                    self.text("match ")?;
                    self.order_call(partial)?;
                    self.text("(&__self_discr,&__arg1_discr)")?;
                    self.order_match(partial)?;
                }
                self.text("match (self,other){")?;
                for row in variants {
                    let selected = &fields[row.fields.clone()];
                    self.pair_pattern(row, selected)?;
                    self.order_fields(selected, true, partial)?;
                    self.text("},")?;
                }
                self.text("_=>")?;
                self.order_equal(partial)?;
                self.text("}")?;
            } else {
                self.order_fields(fields, false, partial)?;
            }
            self.text("} }")?;
        }
        Ok(())
    }
    fn hash_fields(&mut self, fields: &[NodeId], bindings: bool) -> Result<(), LexError> {
        for (index, &field) in fields.iter().enumerate() {
            (self.charge)(1, 0)?;
            self.text("crate::hash::Hash::hash(")?;
            if bindings {
                self.binding("__derive_field_", index)?;
            } else {
                self.text("&")?;
                self.member("self", field, index)?;
            }
            self.text(",state);")?;
        }
        Ok(())
    }
    fn order_fields(
        &mut self,
        fields: &[NodeId],
        bindings: bool,
        partial: bool,
    ) -> Result<(), LexError> {
        for (index, &field) in fields.iter().enumerate() {
            (self.charge)(1, 0)?;
            self.text("match ")?;
            self.order_call(partial)?;
            self.text("(")?;
            if bindings {
                self.binding("__derive_left_", index)?;
            } else {
                self.text("&")?;
                self.member("self", field, index)?;
            }
            self.text(",")?;
            if bindings {
                self.binding("__derive_right_", index)?;
            } else {
                self.text("&")?;
                self.member("other", field, index)?;
            }
            self.text(")")?;
            self.order_match(partial)?;
        }
        self.order_equal(partial)
    }
    fn order_call(&mut self, partial: bool) -> Result<(), LexError> {
        self.text(if partial {
            "crate::cmp::PartialOrd::partial_cmp"
        } else {
            "crate::cmp::Ord::cmp"
        })
    }
    fn order_match(&mut self, partial: bool) -> Result<(), LexError> {
        // A non-equal or unordered field returns immediately before later fields.
        self.text(if partial{" {crate::option::Option::Some(crate::cmp::Ordering::Equal)=>(),__derive_cmp=>return __derive_cmp};"}else{" {crate::cmp::Ordering::Equal=>(),__derive_cmp=>return __derive_cmp};"})
    }
    fn order_equal(&mut self, partial: bool) -> Result<(), LexError> {
        self.text(if partial {
            "crate::option::Option::Some(crate::cmp::Ordering::Equal)"
        } else {
            "crate::cmp::Ordering::Equal"
        })
    }
    fn hasher_name(&self) -> Result<String, LexError> {
        let mut maximum = 0;
        for &node in self.site.ast.list(self.site.item.generics) {
            (self.charge)(1, 0)?;
            let Kind::Generic(index) = self.site.ast.node(node).kind else {
                return Err(error(
                    self.site.original.start,
                    "derived hash lost its actual generic",
                ));
            };
            let name = self.site.ast.generics[index as usize].name;
            let length = name.end.saturating_sub(name.start) as usize;
            maximum = maximum.max(length);
        }
        let length = maximum.checked_add(4).ok_or_else(|| {
            error(
                self.site.original.start,
                "derived hash name exceeds address space",
            )
        })?;
        (self.charge)(length, length.saturating_mul(4))?;
        let mut name = String::new();
        name.try_reserve(length).map_err(|_| {
            error(
                self.site.original.start,
                "derived hash name allocation failed",
            )
        })?;
        name.push_str("__H");
        name.extend(std::iter::repeat_n('_', maximum + 1));
        Ok(name)
    }
    pub(super) fn actual_field(
        &self,
        mut field: NodeId,
    ) -> Result<(Option<super::Span>, NodeId), LexError> {
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
    pub(super) fn member(
        &mut self,
        owner: &str,
        field: NodeId,
        index: usize,
    ) -> Result<(), LexError> {
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
