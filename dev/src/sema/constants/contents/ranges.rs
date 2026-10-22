//! Source contents retain the same checked physical array owner as executable ranges.
use super::{Error, Id, NodeKey, Queries, Query, Value};
use crate::{
    sema::constants::{
        references::Pointer,
        values::{Constant, Reader},
    },
    types::{Kind, Primitive, TypeId},
};
impl Queries<'_> {
    pub(super) fn range_contents(
        &mut self,
        key: NodeKey,
        id: Id,
        base: Id,
        from: u64,
        length: u64,
        value: Value,
    ) -> Result<Value, Error> {
        let query = Query::Contents(key, id);
        let Value::Image(image_id) = value else {
            let facts = &mut *self.typing.facts;
            return facts
                .constant_products
                .range(value, from, length, &facts.texts)
                .map_err(|e| self.error(query, e));
        };
        let owner = self.place(key, Value::Reference(Pointer::new(base)))?.ty;
        let selected = self.place(key, Value::Reference(Pointer::new(id)))?.ty;
        let reader = Reader::new(self.typing.facts);
        let image = reader
            .array_image(Constant(value))
            .map_err(|e| self.error(query, e))?
            .ok_or_else(|| self.error(query, "union byte storage has no structural range view"))?;
        if image.ty != owner
            || from
                .checked_add(length)
                .is_none_or(|end| end > image.bytes.len() as u64)
            || !matches!(self.typing.facts.types.kind(selected), Some(Kind::Array{element,length:n}) if element == TypeId::primitive(Primitive::U8) && n == length)
            || self
                .typing
                .facts
                .layouts
                .get(selected)
                .is_none_or(|layout| layout.size != length)
        {
            return Err(self.error(
                query,
                "constant image range differs from its checked array selection",
            ));
        }
        let work = self
            .typing
            .facts
            .constant_images
            .as_ref()
            .ok_or_else(|| self.error(query, "missing constant image arena"))?
            .range_work(image_id, from, length)
            .map_err(|e| self.error(query, e))?;
        for _ in 0..work {
            self.budget()?;
        }
        let Some(arena) = self.typing.facts.constant_images.as_mut() else {
            return Err(self.error(query, "missing constant image arena"));
        };
        arena
            .array_range(image_id, selected, from, length)
            .map_err(|e| self.error(query, e))
    }
}
