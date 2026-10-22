//! Final reference checks inspect only reachable immutable elements of a selected range.
use super::{Answer, Constant, Inspection, Kind, Origin, TypeId, View};
impl Inspection<'_> {
    pub(super) fn range_reference(
        &mut self,
        ty: TypeId,
        base: crate::sema::values::LocationId,
        from: u64,
        length: u64,
    ) -> Result<Answer, &'static str> {
        let parent = self.reader.location(base)?;
        let Some(Kind::Array {
            element,
            length: bound,
        }) = self.reader.facts.types.kind(parent.ty)
        else {
            return Err("constant range backing is not a canonical array");
        };
        match self.reader.facts.types.kind(ty) {
            Some(Kind::Array {
                element: actual,
                length: n,
            }) if self.reader.same_storage_type(actual, element) && n == length => {}
            Some(Kind::Slice(actual)) if self.reader.same_storage_type(actual, element) => {}
            _ => return Err("constant range referent differs from its selected element type"),
        }
        if from.checked_add(length).is_none_or(|end| end > bound) {
            return Err("constant range reference exceeds its backing");
        }
        if length == 0 {
            return Ok(Answer::Ordinary);
        }
        let value = self.completed_backing(base)?;
        match self.reader.view(value)? {
            View::Repeat {
                value,
                length: actual,
            } if actual == bound => self.range_element(element, value),
            View::Product {
                length: actual,
                variant: None,
            } if actual == bound => {
                let mut unresolved = false;
                self.charge(
                    usize::try_from(length).map_err(|_| "constant range exceeds host size")?,
                )?;
                for index in from..from + length {
                    match self.range_element(element, self.reader.field(value, index)?)? {
                        Answer::Interior => return Ok(Answer::Interior),
                        Answer::Unresolved => unresolved = true,
                        Answer::Ordinary => {}
                    }
                }
                Ok(if unresolved {
                    Answer::Unresolved
                } else {
                    Answer::Ordinary
                })
            }
            View::Bytes(_) | View::ByteRange { .. }
                if element == TypeId::primitive(crate::types::Primitive::U8) =>
            {
                self.reader.stored_bytes(parent.ty, value)?;
                Ok(Answer::Ordinary)
            }
            _ => Err("constant range backing differs from its stored array"),
        }
    }
    fn range_element(&mut self, ty: TypeId, value: Constant) -> Result<Answer, &'static str> {
        let (answer, work) = self.reader.shared_mutation_work(ty, value)?;
        self.charge(work)?;
        if answer == Answer::Ordinary {
            self.push(ty, value)?;
        }
        Ok(answer)
    }
    fn completed_backing(
        &mut self,
        mut location: crate::sema::values::LocationId,
    ) -> Result<Constant, &'static str> {
        let mut fields = Vec::new();
        let mut value = loop {
            self.charge(1)?;
            let selected = self.reader.location(location)?;
            if let Some(value) = selected.value {
                break value;
            }
            match selected.origin {
                Origin::Projection { base, index } => {
                    fields.push(index);
                    location = base;
                }
                _ => return Err("constant reference backing is unfinished"),
            }
        };
        for index in fields.into_iter().rev() {
            self.charge(1)?;
            value = self.reader.field(value, index)?;
        }
        Ok(value)
    }
}
