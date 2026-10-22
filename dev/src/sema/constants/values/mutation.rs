//! Evaluated storage refines the conservative type fact without following pointer slots.
mod ranges;
use super::{Constant, Error, Evaluator, Origin, Reader, View};
use crate::types::{
    Kind, TypeId,
    aggregate::{ShapeReadiness, SharedMutation},
    interior_mutability::{Answer, Query},
};
use std::collections::HashSet;

struct Inspection<'a> {
    reader: Reader<'a>,
    types: Query<'a>,
    pending: Vec<(TypeId, Constant)>,
    seen: HashSet<(TypeId, Constant)>,
    steps: usize,
    references: bool,
}
impl Reader<'_> {
    /// Refine owned storage using its actual array length and active variant.
    /// References and raw pointers describe their own slots, not their referents.
    /// This fact grants no auto trait, const write, or ordinary reference write.
    /// # Errors
    /// Rejects malformed typed values and exhausted storage-graph work.
    pub fn shared_mutation(self, ty: TypeId, value: Constant) -> Result<Answer, &'static str> {
        self.shared_mutation_work(ty, value)
            .map(|(answer, _)| answer)
    }
    pub(crate) fn shared_mutation_work(
        self,
        ty: TypeId,
        value: Constant,
    ) -> Result<(Answer, usize), &'static str> {
        self.inspect_storage(ty, value, false)
    }
    /// Check escaped references independently of owned storage and raw addresses.
    /// Named static backing is permitted by its actual symbolic origin. An owned
    /// value may contain interior storage, but a reference to a fresh allocation
    /// containing that storage cannot escape constant evaluation.
    /// # Errors
    /// Rejects malformed references, unfinished backing and exhausted graph work.
    pub fn shared_reference_storage(
        self,
        ty: TypeId,
        value: Constant,
    ) -> Result<Answer, &'static str> {
        self.shared_reference_work(ty, value)
            .map(|(answer, _)| answer)
    }
    pub(crate) fn shared_reference_work(
        self,
        ty: TypeId,
        value: Constant,
    ) -> Result<(Answer, usize), &'static str> {
        self.inspect_storage(ty, value, true)
    }
    fn inspect_storage(
        self,
        ty: TypeId,
        value: Constant,
        references: bool,
    ) -> Result<(Answer, usize), &'static str> {
        if !self.facts.aggregates.has_shared_mutation() {
            return Ok((Answer::Ordinary, 0));
        }
        let mut inspection = Inspection {
            reader: self,
            types: Query::new(&self.facts.types, &self.facts.aggregates),
            pending: vec![(ty, value)],
            seen: HashSet::new(),
            steps: 0,
            references,
        };
        let answer = inspection.inspect()?;
        Ok((answer, inspection.steps + inspection.types.work().steps))
    }
}
impl Evaluator<'_> {
    pub(super) fn checked_reference_result(
        &mut self,
        query: super::Query,
        ty: TypeId,
        value: Constant,
    ) -> Result<Constant, Error> {
        if !self.facts().aggregates.has_shared_mutation() {
            return Ok(value);
        }
        let (answer, work) = Reader::new(self.facts())
            .shared_reference_work(ty, value)
            .map_err(|e| self.queries.error(query, e))?;
        self.queries.steps = self
            .queries
            .steps
            .checked_add(work)
            .filter(|steps| *steps <= self.queries.limit)
            .ok_or_else(|| {
                self.queries
                    .error(query, "constant reference storage work budget exceeded")
            })?;
        match answer {
            Answer::Ordinary => Ok(value),
            Answer::Interior => Err(self.queries.error(
                query,
                "constant reference cannot escape with interior mutable allocation backing",
            )),
            Answer::Unresolved => Err(self.queries.error(
                query,
                "constant reference storage requires completed type and value facts",
            )),
        }
    }
}
impl Inspection<'_> {
    fn inspect(&mut self) -> Result<Answer, &'static str> {
        let mut unresolved = false;
        while let Some((ty, value)) = self.pending.pop() {
            self.charge(1)?;
            if !self.seen.insert((ty, value)) {
                continue;
            }
            if !self.references && self.types.contains(ty)? == Answer::Ordinary {
                continue;
            }
            match self
                .reader
                .facts
                .types
                .kind(ty)
                .ok_or("invalid stored value type")?
            {
                Kind::Adt(item, _) => match self.nominal(item, ty, value)? {
                    Answer::Interior => return Ok(Answer::Interior),
                    Answer::Unresolved => unresolved = true,
                    Answer::Ordinary => {}
                },
                Kind::Array { element, length } => self.array(element, length, value)?,
                Kind::Slice(element) => {
                    let length = self.reader.field_types(ty, value)?.len();
                    self.array(element, length, value)?;
                }
                Kind::Tuple(fields) => self.fields(fields, value)?,
                Kind::Closure { environment, .. } => self.push(environment, value)?,
                Kind::Reference { .. } if self.references => match self.reference(value)? {
                    Answer::Interior => return Ok(Answer::Interior),
                    Answer::Unresolved => unresolved = true,
                    Answer::Ordinary => {}
                },
                Kind::ArrayTerm { .. }
                | Kind::Parameter(_)
                | Kind::Projection(_)
                | Kind::Opaque(..)
                | Kind::Dynamic { .. } => unresolved = true,
                Kind::Never
                | Kind::Primitive(_)
                | Kind::Pointer { .. }
                | Kind::Reference { .. }
                | Kind::Function { .. }
                | Kind::FunctionItem { .. } => {}
            }
        }
        Ok(if unresolved {
            Answer::Unresolved
        } else {
            Answer::Ordinary
        })
    }
    fn nominal(
        &mut self,
        item: crate::types::ItemId,
        ty: TypeId,
        value: Constant,
    ) -> Result<Answer, &'static str> {
        if !self.references
            && self.reader.facts.aggregates.shared_mutation(item) == Some(SharedMutation::Interior)
        {
            return Ok(Answer::Interior);
        }
        let ShapeReadiness::Ready(shape) = self
            .reader
            .facts
            .aggregates
            .try_shape(&self.reader.facts.types, ty)?
        else {
            return Ok(Answer::Unresolved);
        };
        if let View::Image { .. } = self.reader.view(value)? {
            if shape.definition.kind != crate::types::aggregate::Kind::Union
                || self.reader.image(value)?.ty != ty
            {
                return Err("constant union image differs from its nominal storage");
            }
            // The image arena only admits all-fields primitive scalar families.
            return Ok(Answer::Ordinary);
        }
        let View::Product { variant, .. } = self.reader.view(value)? else {
            return Err("nominal storage lacks a product value");
        };
        let ordinal = if let Some(variant) = variant {
            let variant = self
                .reader
                .facts
                .members
                .variants
                .get(variant.0 as usize)
                .ok_or("invalid stored value variant")?;
            if variant.owner != item {
                return Err("stored value variant differs from its nominal owner");
            }
            variant.ordinal as usize
        } else if shape.variants().len() == 1 {
            0
        } else {
            return Err("enum storage lacks an active variant");
        };
        let fields = shape
            .variants()
            .nth(ordinal)
            .ok_or("stored value variant exceeds its shape")?
            .fields;
        self.fields(fields, value)?;
        Ok(Answer::Ordinary)
    }
    fn reference(&mut self, value: Constant) -> Result<Answer, &'static str> {
        let mut location = match self.reader.view(value)? {
            View::Reference(id) => id,
            View::Text(_) | View::Bytes(_) => return Ok(Answer::Ordinary),
            _ => return Err("constant reference lacks symbolic backing"),
        };
        let pointed = self.reader.location(location)?;
        loop {
            self.charge(1)?;
            match self.reader.location(location)?.origin {
                Origin::Projection { base, .. } | Origin::Subsequence { base, .. } => {
                    location = base;
                }
                Origin::Static(_) => return Ok(Answer::Ordinary),
                Origin::Allocation => break,
            }
        }
        if let Origin::Subsequence { base, from, length } = pointed.origin {
            return self.range_reference(pointed.ty, base, from, length);
        }
        let value = pointed
            .value
            .ok_or("constant reference backing is unfinished")?;
        let (answer, work) = self.reader.shared_mutation_work(pointed.ty, value)?;
        self.charge(work)?;
        if answer == Answer::Ordinary {
            self.push(pointed.ty, value)?;
        }
        Ok(answer)
    }
    fn fields(
        &mut self,
        fields: crate::types::ListId,
        value: Constant,
    ) -> Result<(), &'static str> {
        let fields = self
            .reader
            .facts
            .types
            .list(fields)
            .ok_or("invalid stored value fields")?;
        let View::Product { length, .. } = self.reader.view(value)? else {
            return Err("stored fields lack a product value");
        };
        if usize::try_from(length).ok() != Some(fields.len()) {
            return Err("stored product arity differs from its field types");
        }
        for (index, &ty) in fields.iter().enumerate() {
            self.push(ty, self.reader.field(value, index as u64)?)?;
        }
        Ok(())
    }
    fn array(&mut self, element: TypeId, length: u64, value: Constant) -> Result<(), &'static str> {
        match self.reader.view(value)? {
            View::Repeat {
                value,
                length: actual,
            } if actual == length => {
                if length != 0 {
                    self.push(element, value)?;
                }
            }
            View::Product {
                length: actual,
                variant: None,
            } if actual == length => {
                self.charge(
                    usize::try_from(length).map_err(|_| "stored array exceeds host size")?,
                )?;
                for index in 0..length {
                    self.push(element, self.reader.field(value, index)?)?;
                }
            }
            View::Bytes(_) | View::ByteRange { .. }
                if element == TypeId::primitive(crate::types::Primitive::U8) =>
            {
                let actual = match self.reader.view(value)? {
                    View::Bytes(text) => self.reader.bytes(text)?.len() as u64,
                    View::ByteRange { length, .. } => length,
                    _ => return Err("missing byte range"),
                };
                if actual != length {
                    return Err("stored byte range differs from its type length");
                }
            }
            _ => return Err("stored array value differs from its type length"),
        }
        Ok(())
    }
    fn push(&mut self, ty: TypeId, value: Constant) -> Result<(), &'static str> {
        self.charge(1)?;
        self.pending.push((ty, value));
        Ok(())
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|steps| *steps <= 4_000_000)
            .ok_or("evaluated shared mutation work budget exceeded")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/evaluated_storage.rs"]
mod tests;
