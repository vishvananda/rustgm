//! Complete evaluator values become canonical const DAGs using already published field recipes.
use crate::{
    sema::{
        constants::{
            Value,
            values::{Constant, FieldRecipe, Reader, View},
        },
        header_attributes,
        model::{Declarations, Scalar},
    },
    support::{hash::HashMap, retained::Growth},
    types::{
        Kind, Primitive, TypeId, aggregate,
        arguments::{ConstId, ConstKind, Constant as Bits},
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ty: TypeId,
    value: Constant,
}
#[derive(Clone, Copy)]
enum Recipe {
    Fields {
        types: FieldRecipe,
        length: u64,
        variant: Option<u32>,
    },
    Repeat(Key),
}
enum Task {
    Enter(Key),
    Finish(Key, Recipe),
}
struct Budget<'a> {
    steps: &'a mut usize,
    limit: usize,
    bytes: usize,
}
impl Budget<'_> {
    fn local(&mut self, work: usize, bytes: usize) -> Result<(), &'static str> {
        *self.steps = self.steps.saturating_add(work);
        self.bytes = self.bytes.saturating_add(bytes);
        if *self.steps > self.limit || self.bytes > crate::support::crates::MAX_BYTES {
            return Err("canonical const argument work or storage budget exceeded");
        }
        Ok(())
    }
    fn charge(
        &mut self,
        facts: &Declarations,
        work: usize,
        bytes: usize,
    ) -> Result<(), &'static str> {
        self.local(work, bytes)?;
        if facts.source_bindings.is_some() {
            header_attributes::source_payments(facts.must_use_contracts.as_deref())
                .map_err(|_| "canonical const lacks its completed source payer")?
                .charge(work, bytes)?;
        }
        Ok(())
    }
}
struct Conversion<'a, 'b> {
    facts: &'a mut Declarations,
    budget: Budget<'b>,
    ready: HashMap<Key, Option<ConstId>>,
    pending: Vec<Task>,
}
pub(super) fn canonicalize(
    facts: &mut Declarations,
    steps: &mut usize,
    limit: usize,
    ty: TypeId,
    value: Constant,
) -> Result<ConstId, &'static str> {
    if let Value::Scalar(scalar) = value.0 {
        return scalar_value(facts, ty, scalar);
    }
    if let Value::Canonical(id) = value.0 {
        Budget {
            steps,
            limit,
            bytes: 0,
        }
        .charge(facts, 1, 0)?;
        if facts
            .types
            .aggregate_constant(id)
            .is_none_or(|value| value.ty() != ty)
        {
            return Err("canonical argument differs from its exact completed type");
        }
        return facts.types.intern_const(ConstKind::Aggregate(id));
    }
    let mut conversion = Conversion {
        facts,
        budget: Budget {
            steps,
            limit,
            bytes: 0,
        },
        ready: HashMap::default(),
        pending: Vec::new(),
    };
    let root = Key { ty, value };
    conversion.push(Task::Enter(root))?;
    while let Some(task) = conversion.pending.pop() {
        conversion.budget.charge(conversion.facts, 1, 0)?;
        match task {
            Task::Enter(key) => conversion.enter(key)?,
            Task::Finish(key, recipe) => conversion.finish(key, recipe)?,
        }
    }
    conversion.completed(root)
}
fn scalar_value(
    facts: &mut Declarations,
    ty: TypeId,
    scalar: Scalar,
) -> Result<ConstId, &'static str> {
    let kind = scalar_kind(&facts.types, ty, scalar)?;
    facts.types.intern_const(kind)
}
fn scalar_kind(
    types: &crate::types::Store,
    ty: TypeId,
    scalar: Scalar,
) -> Result<ConstKind, &'static str> {
    let bits = match (types.kind(ty), scalar) {
        (Some(Kind::Primitive(Primitive::Bool)), Scalar::Boolean(value)) => u128::from(value),
        (Some(Kind::Primitive(Primitive::Char)), Scalar::Character(value)) => {
            u128::from(u32::from(value))
        }
        (Some(Kind::Primitive(p)), Scalar::Integer(bits)) if p.integer().is_some() => bits,
        _ => return Err("const argument scalar differs from its exact primitive type"),
    };
    Ok(ConstKind::Value(Bits { ty, bits }))
}
impl Conversion<'_, '_> {
    fn push(&mut self, task: Task) -> Result<(), &'static str> {
        let growth = Growth::vector::<Task>(self.pending.len(), self.pending.capacity(), 1);
        self.budget.charge(self.facts, growth.work, growth.bytes)?;
        self.pending.push(task);
        Ok(())
    }
    fn completed(&self, key: Key) -> Result<ConstId, &'static str> {
        self.ready
            .get(&key)
            .copied()
            .flatten()
            .ok_or("canonical const child was not completed")
    }
    fn enter(&mut self, key: Key) -> Result<(), &'static str> {
        if let Some(state) = self.ready.get(&key) {
            return if state.is_some() {
                Ok(())
            } else {
                Err("cycle in canonical const value dependencies")
            };
        }
        let growth =
            Growth::map::<Key, Option<ConstId>>(self.ready.len(), self.ready.capacity(), 1);
        self.budget.charge(self.facts, growth.work, growth.bytes)?;
        self.ready.insert(key, None);
        if let Value::Scalar(scalar) = key.value.0 {
            let kind = scalar_kind(&self.facts.types, key.ty, scalar)?;
            let payments = self
                .facts
                .source_bindings
                .as_ref()
                .map(|_| {
                    header_attributes::source_payments(self.facts.must_use_contracts.as_deref())
                })
                .transpose()
                .map_err(|_| "canonical const lacks its completed source payer")?;
            let budget = &mut self.budget;
            let id = self.facts.types.intern_const_charged(kind, |work, bytes| {
                budget.local(work, bytes)?;
                if let Some(payer) = &payments {
                    payer.charge(work, bytes)?;
                }
                Ok(())
            })?;
            self.ready.insert(key, Some(id));
            return Ok(());
        }
        if let Value::Canonical(id) = key.value.0 {
            if self
                .facts
                .types
                .aggregate_constant(id)
                .is_none_or(|value| value.ty() != key.ty)
            {
                return Err("canonical argument differs from its exact completed type");
            }
            let id = self.facts.types.intern_const(ConstKind::Aggregate(id))?;
            self.ready.insert(key, Some(id));
            return Ok(());
        }
        let recipe = self.recipe(key)?;
        self.push(Task::Finish(key, recipe))?;
        match recipe {
            Recipe::Repeat(child) => self.push(Task::Enter(child))?,
            Recipe::Fields { types, length, .. } => {
                self.budget.charge(
                    self.facts,
                    usize::try_from(length).map_err(|_| "const arity exceeds host")?,
                    0,
                )?;
                for index in (0..length).rev() {
                    let child = Key {
                        ty: types
                            .get(&self.facts.types, index)
                            .ok_or("missing canonical field type")?,
                        value: Reader::new(self.facts).field(key.value, index)?,
                    };
                    self.push(Task::Enter(child))?;
                }
            }
        }
        Ok(())
    }
    fn recipe(&self, key: Key) -> Result<Recipe, &'static str> {
        let reader = Reader::new(self.facts);
        let length = match reader.view(key.value)? {
            View::Repeat { value, length } => {
                let Some(Kind::Array {
                    element,
                    length: expected,
                }) = self.facts.types.kind(key.ty)
                else {
                    return Err("const repeat requires its complete fixed-array type");
                };
                if length != expected {
                    return Err("const repeat differs from its actual array length");
                }
                if length == 0 {
                    return Ok(Recipe::Fields {
                        types: FieldRecipe::Repeated { element, length },
                        length,
                        variant: None,
                    });
                }
                return Ok(Recipe::Repeat(Key { ty: element, value }));
            }
            View::Product { length, .. } | View::ByteRange { length, .. } => length,
            View::Bytes(text) => reader.bytes(text)?.len() as u64,
            View::Image { .. } => {
                reader.require_publishable_array(key.value)?;
                let image = reader
                    .array_image(key.value)?
                    .ok_or("const image requires its structural array profile")?;
                if image.ty != key.ty {
                    return Err("const image differs from its exact array type");
                }
                image.bytes.len() as u64
            }
            _ => return Err("const argument requires its complete structural value profile"),
        };
        let types = reader.field_recipe(key.ty, key.value)?;
        if types.len(&self.facts.types)? != length {
            return Err("const product differs from its completed field arity");
        }
        let variant = self.variant(key)?;
        Ok(Recipe::Fields {
            types,
            length,
            variant,
        })
    }
    fn variant(&self, key: Key) -> Result<Option<u32>, &'static str> {
        let Some(Kind::Adt(item, _)) = self.facts.types.kind(key.ty) else {
            return Ok(None);
        };
        let definition = self
            .facts
            .aggregates
            .get(item)
            .ok_or("const nominal lost its declaration")?;
        match definition.kind {
            aggregate::Kind::Struct | aggregate::Kind::PhantomData => Ok(None),
            aggregate::Kind::Enum => {
                let View::Product {
                    variant: Some(id), ..
                } = Reader::new(self.facts).view(key.value)?
                else {
                    return Err("const enum lost its actual active variant");
                };
                let member = self
                    .facts
                    .members
                    .variants
                    .get(id.0 as usize)
                    .ok_or("missing const enum variant")?;
                if member.owner != item {
                    return Err("const enum variant belongs to another owner");
                }
                Ok(Some(member.ordinal))
            }
            aggregate::Kind::Union => Err("union bytes are not a structural const argument"),
        }
    }
    fn finish(&mut self, key: Key, recipe: Recipe) -> Result<(), &'static str> {
        let mut children = Vec::new();
        let (variant, repeated) = match recipe {
            Recipe::Repeat(child) => (None, Some(self.completed(child)?)),
            Recipe::Fields {
                types,
                length,
                variant,
            } => {
                let count = usize::try_from(length)
                    .map_err(|_| "canonical const field count exceeds host")?;
                let growth = Growth::vector::<ConstId>(0, 0, count);
                self.budget
                    .charge(self.facts, growth.work.saturating_add(count), growth.bytes)?;
                children
                    .try_reserve(count)
                    .map_err(|_| "canonical const field allocation failed")?;
                for index in 0..length {
                    let child = Key {
                        ty: types
                            .get(&self.facts.types, index)
                            .ok_or("missing canonical field type")?,
                        value: Reader::new(self.facts).field(key.value, index)?,
                    };
                    children.push(self.completed(child)?);
                }
                (variant, None)
            }
        };
        let payments = self
            .facts
            .source_bindings
            .as_ref()
            .map(|_| header_attributes::source_payments(self.facts.must_use_contracts.as_deref()))
            .transpose()
            .map_err(|_| "canonical const lacks its completed source payer")?;
        let budget = &mut self.budget;
        let pay = |work, bytes| {
            budget.local(work, bytes)?;
            if let Some(payer) = &payments {
                payer.charge(work, bytes)?;
            }
            Ok(())
        };
        let id = if let Some(element) = repeated {
            self.facts
                .types
                .intern_repeated_constant(key.ty, element, pay)?
        } else {
            self.facts
                .types
                .intern_aggregate_constant(key.ty, variant, &children, pay)?
        };
        self.ready.insert(key, Some(id));
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/valtree_inputs.rs"]
mod tests;
