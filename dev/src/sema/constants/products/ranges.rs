//! Immutable ranges share field or byte arenas and retain compressed repeat recipes.
use super::{Contents, Id, Products, Value};
impl Products {
    pub(in crate::sema::constants) fn view(
        &self,
        id: Id,
    ) -> Result<crate::sema::values::View, &'static str> {
        use crate::sema::values::{Constant, View};
        Ok(
            match *self
                .nodes
                .get(id.0 as usize)
                .ok_or("missing constant product")?
            {
                Contents::Fields {
                    start,
                    end,
                    variant,
                } => View::Product {
                    length: u64::from(end - start),
                    variant,
                },
                Contents::Repeat { value, length } => View::Repeat {
                    value: Constant(value),
                    length,
                },
                Contents::Bytes { text, from, length } => View::ByteRange { text, from, length },
            },
        )
    }
    pub(in crate::sema::constants) fn byte_range(
        &self,
        id: Id,
    ) -> Result<Option<(crate::sema::text::Id, u64, u64)>, &'static str> {
        Ok(
            match *self
                .nodes
                .get(id.0 as usize)
                .ok_or("missing constant product")?
            {
                Contents::Bytes { text, from, length } => Some((text, from, length)),
                _ => None,
            },
        )
    }
    pub(in crate::sema::constants) fn range(
        &mut self,
        value: Value,
        from: u64,
        length: u64,
        texts: &crate::sema::text::Texts,
    ) -> Result<Value, &'static str> {
        let (contents, bound) = match value {
            Value::Product(id) => {
                let bound = self.length(id)?;
                let contents = match *self
                    .nodes
                    .get(id.0 as usize)
                    .ok_or("missing constant product")?
                {
                    Contents::Fields {
                        start,
                        variant: None,
                        ..
                    } => {
                        let start = u32::try_from(
                            u64::from(start)
                                .checked_add(from)
                                .ok_or("constant range offset overflow")?,
                        )
                        .map_err(|_| "constant field range exceeds storage")?;
                        let end = u32::try_from(
                            u64::from(start)
                                .checked_add(length)
                                .ok_or("constant range offset overflow")?,
                        )
                        .map_err(|_| "constant field range exceeds storage")?;
                        Contents::Fields {
                            start,
                            end,
                            variant: None,
                        }
                    }
                    Contents::Repeat { value, .. } => Contents::Repeat { value, length },
                    Contents::Bytes {
                        text, from: offset, ..
                    } => Contents::Bytes {
                        text,
                        from: offset
                            .checked_add(from)
                            .ok_or("constant byte range offset overflow")?,
                        length,
                    },
                    Contents::Fields { .. } => {
                        return Err("constant sequence range requires nonnominal storage");
                    }
                };
                (contents, bound)
            }
            Value::Bytes(text) => (
                Contents::Bytes { text, from, length },
                texts.get(text)?.len() as u64,
            ),
            _ => return Err("constant sequence range requires sequence storage"),
        };
        if from.checked_add(length).is_none_or(|end| end > bound) {
            return Err("constant range is out of bounds");
        }
        // Recipe headers share all payload storage. Include geometric Vec slack.
        if self.nodes.len().saturating_add(2) > 64 * 1024 * 1024 / 128 {
            return Err("constant sequence recipe storage limit exceeded");
        }
        self.insert(contents)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rest_product_recipe_keeps_ordinary_node_density() {
        assert_eq!(std::mem::size_of::<super::super::Contents>(), 48);
    }
    #[test]
    fn rest_byte_ranges_retain_only_recipe_headers() -> Result<(), &'static str> {
        for width in [8, 1024, 1_000_000] {
            let mut texts = crate::sema::text::Texts::default();
            let text = texts.intern(&vec![7; width])?;
            let mut products = super::Products::default();
            let first = products.range(super::Value::Bytes(text), 1, width as u64 - 2, &texts)?;
            let second = products.range(first, 1, width as u64 - 4, &texts)?;
            let super::Value::Product(id) = second else {
                return Err("missing range recipe");
            };
            assert_eq!(products.fields.len(), 0);
            assert_eq!(products.nodes.len(), 2);
            assert_eq!(products.byte_range(id)?, Some((text, 2, width as u64 - 4)));
            assert!(matches!(
                products.get(id, width as u64 - 5, &texts)?,
                super::Value::Scalar(crate::sema::model::Scalar::Integer(7))
            ));
        }
        Ok(())
    }
}
