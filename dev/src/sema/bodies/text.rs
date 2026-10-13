//! Text types and byte-string patterns consume already decoded literal facts.
use super::{Checker, Constructor, Error, NodeId, TermId, Type, TypeId};
use crate::{
    sema::patterns::{Kind as Pattern, PatternId},
    types::{Primitive, RegionId},
};

impl Checker<'_> {
    pub(super) fn text_literal_type(
        &mut self,
        node: NodeId,
        bytes: &[u8],
        string: bool,
    ) -> Result<TermId, Error> {
        let text = self
            .facts
            .texts
            .intern(bytes)
            .map_err(|e| self.error(node, e))?;
        self.facts.text_literals.insert(self.key(node), text);
        let referent = if string {
            TypeId::primitive(Primitive::Str)
        } else {
            let length = u64::try_from(bytes.len())
                .map_err(|_| self.error(node, "byte string length overflow"))?;
            self.facts
                .types
                .intern(Type::Array {
                    element: TypeId::primitive(Primitive::U8),
                    length,
                })
                .map_err(|e| self.error(node, e))?
        };
        let ty = self
            .facts
            .types
            .intern(Type::Reference {
                mutable: false,
                region: RegionId::STATIC,
                referent,
            })
            .map_err(|e| self.error(node, e))?;
        self.known(node, ty)
    }
    pub(super) fn is_text_literal(&self, node: NodeId, index: u32) -> Result<bool, Error> {
        use crate::literal::storage::StoredValue;
        let literal = self
            .report
            .package
            .unit(self.file)
            .configured
            .ast
            .literals
            .get(index as usize)
            .ok_or_else(|| self.error(node, "missing literal identity"))?;
        Ok(matches!(
            literal,
            StoredValue::String(_) | StoredValue::ByteString(_) | StoredValue::CString(_)
        ))
    }
    pub(super) fn text_pattern(
        &mut self,
        node: NodeId,
        index: u32,
        term: TermId,
        id: PatternId,
    ) -> Result<Pattern, Error> {
        let literal = self.literal(node, index)?;
        // Byte-string literal patterns alone permit an expected &[u8] slice.
        // A named constant preserves its declared array/reference type.
        let literal = self
            .array_reference_slice(node, literal, term)?
            .unwrap_or(literal);
        self.equate_pattern(node, literal, term)?;
        self.request_pattern_literal(node, id);
        self.pattern_refutable = true;
        Ok(Pattern::Pending)
    }
    pub(super) fn array_reference_slice(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<Option<TermId>, Error> {
        let (
            Some(Constructor::Reference { mutable: a, region }),
            Some(Constructor::Reference { mutable: b, .. }),
        ) = (self.head(node, actual)?, self.head(node, expected)?)
        else {
            return Ok(None);
        };
        if a != b {
            return Ok(None);
        }
        let source = self.child(node, actual, 0)?;
        let target = self.child(node, expected, 0)?;
        if !matches!(self.head(node, source)?, Some(Constructor::Array(_)))
            || self.head(node, target)? != Some(Constructor::Slice)
        {
            return Ok(None);
        }
        let element = self.child(node, source, 0)?;
        let slice = self
            .infer
            .construct(Constructor::Slice, &[element])
            .map_err(|e| self.error(node, e))?;
        let reference = self
            .infer
            .construct(Constructor::Reference { mutable: a, region }, &[slice])
            .map_err(|e| self.error(node, e))?;
        Ok(Some(reference))
    }
}
