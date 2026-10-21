//! Evaluated storage refines the conservative type fact without following pointer slots.
use super::{Constant, Reader, View};
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
        if !self.facts.aggregates.has_shared_mutation() {
            return Ok((Answer::Ordinary, 0));
        }
        let mut inspection = Inspection {
            reader: self,
            types: Query::new(&self.facts.types, &self.facts.aggregates),
            pending: vec![(ty, value)],
            seen: HashSet::new(),
            steps: 0,
        };
        let answer = inspection.inspect()?;
        Ok((answer, inspection.steps + inspection.types.work().steps))
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
            if self.types.contains(ty)? == Answer::Ordinary {
                continue;
            }
            match self
                .reader
                .facts
                .types
                .kind(ty)
                .ok_or("invalid stored value type")?
            {
                Kind::Adt(item, _) => {
                    if self.reader.facts.aggregates.shared_mutation(item)
                        == Some(SharedMutation::Interior)
                    {
                        return Ok(Answer::Interior);
                    }
                    let ShapeReadiness::Ready(shape) = self
                        .reader
                        .facts
                        .aggregates
                        .try_shape(&self.reader.facts.types, ty)?
                    else {
                        unresolved = true;
                        continue;
                    };
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
                }
                Kind::Array { element, length } => self.array(element, length, value)?,
                Kind::Tuple(fields) => self.fields(fields, value)?,
                Kind::Closure { environment, .. } => self.push(environment, value)?,
                Kind::ArrayTerm { .. }
                | Kind::Slice(_)
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
