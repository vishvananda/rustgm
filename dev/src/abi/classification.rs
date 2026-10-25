//! One cached x86-64 C boundary plan derives classes from prepared typed layouts.
mod composites;
#[cfg(test)]
#[path = "../../tests/invariants/composite_classes.rs"]
mod tests;
use super::layout::{Layouts, Shape};
use crate::support::hash::HashMap;
use crate::types::{
    self, Kind, Store, TypeId,
    aggregate::{Definitions, Representation},
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Integer,
    Sse,
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classes {
    Eightbytes { first: Class, second: Option<Class> },
    Memory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    pub bytes: u64,
    pub align: u32,
    pub classes: Classes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passing {
    Scalar(TypeId),
    Object(Object),
}
pub struct Plan {
    pub signature: TypeId,
    pub parameters: Box<[Passing]>,
    pub result: Passing,
}
#[derive(Default)]
pub struct Plans {
    signatures: HashMap<TypeId, Arc<Plan>>,
    types: HashMap<TypeId, Result<Passing, &'static str>>,
    composites: Option<Box<composites::Cache>>,
    pub steps: usize,
    pub retained_bytes: usize,
}
impl Plans {
    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        account_work(&mut self.steps, work)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        storage(&mut self.retained_bytes, bytes)
    }
    /// # Errors
    /// Rejects unsupported calling conventions, unresolved shapes and aggregate profiles.
    pub fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        signature: TypeId,
    ) -> Result<Arc<Plan>, &'static str> {
        self.charge(1)?;
        if let Some(plan) = self.signatures.get(&signature) {
            return Ok(Arc::clone(plan));
        }
        let Some(Kind::Function {
            parameters,
            result,
            convention,
            ..
        }) = types.kind(signature)
        else {
            return Err("C ABI plan requires a function signature");
        };
        if convention.abi != types::callable::Abi::C {
            return Err("C ABI plan requires a C convention");
        }
        let parameters = types.list(parameters).ok_or("missing C ABI inputs")?;
        self.retain(
            parameters
                .len()
                .checked_mul(size_of::<Passing>())
                .and_then(|n| n.checked_add(96))
                .ok_or("C ABI plan storage overflow")?,
        )?;
        let mut passing = Vec::with_capacity(parameters.len());
        for &ty in parameters {
            passing.push(self.passing(types, definitions, layouts, ty)?);
        }
        let result = self.passing(types, definitions, layouts, result)?;
        let plan = Arc::new(Plan {
            signature,
            parameters: passing.into_boxed_slice(),
            result,
        });
        self.signatures.insert(signature, Arc::clone(&plan));
        Ok(plan)
    }
    /// Classify one actual argument using the same immutable type/layout cache.
    /// # Errors
    /// Rejects unsupported or unresolved C boundary types and exhausted budgets.
    pub fn passing(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        ty: TypeId,
    ) -> Result<Passing, &'static str> {
        self.charge(1)?;
        if let Some(&passing) = self.types.get(&ty) {
            return passing;
        }
        self.retain(64)?;
        let value = self.classify(types, definitions, layouts, ty);
        self.types.insert(ty, value);
        value
    }
    fn classify(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        ty: TypeId,
    ) -> Result<Passing, &'static str> {
        let head = types.storage_head(ty)?;
        let id = layouts
            .demand(types, definitions, head)
            .map_err(|e| e.message)?;
        let layout = *layouts.at(id).ok_or("missing C ABI layout")?;
        let kind = types.kind(head).ok_or("missing C ABI type")?;
        if matches!(
            kind,
            Kind::Primitive(types::Primitive::F16 | types::Primitive::F128)
        ) {
            return Err("wide floating C ABI is not implemented");
        }
        if head == TypeId::UNIT
            || head == TypeId::NEVER
            || matches!(kind, Kind::FunctionItem { .. })
        {
            return Ok(Passing::Scalar(ty));
        }
        if matches!(layout.shape, Shape::Scalar) && scalar(kind) {
            return Ok(Passing::Scalar(ty));
        }
        let Kind::Adt(item, _) = kind else {
            return Err("C aggregate ABI profile requires a repr(C) scalar-field struct");
        };
        let definition = definitions
            .get(item)
            .ok_or("missing C aggregate declaration")?;
        if let Some(niche) = layouts.niche(&layout)? {
            if niche.owner != head {
                return Err("nullable C ABI encoding belongs to another type");
            }
            return Ok(Passing::Scalar(niche.word));
        }
        if definition.repr.representation == Representation::Transparent
            && let Some(carrier) = layouts.pointer_carrier(types, definitions, head)?
        {
            return Ok(Passing::Scalar(carrier));
        }
        if definition.kind != types::aggregate::Kind::Struct
            || definition.repr.representation != Representation::C
            || definition.repr.packed.is_some()
            || layout.size == 0
        {
            return Err("C aggregate ABI profile requires a nonempty aligned repr(C) struct");
        }
        let variant = *layouts
            .variants(&layout)
            .first()
            .ok_or("missing C aggregate field layout")?;
        let fields = types
            .list(variant.fields)
            .ok_or("missing C aggregate fields")?;
        let offsets = layouts.offsets(&variant);
        let mut classes = [Class::None; 2];
        for (index, &field) in fields.iter().enumerate() {
            self.charge(1)?;
            let field = types.storage_head(field)?;
            let kind = types.kind(field).ok_or("missing C field type")?;
            let child = layouts.get(field).ok_or("missing C field layout")?;
            if !scalar(kind) || !matches!(child.shape, Shape::Scalar) || child.size == 0 {
                return self.composite(types, definitions, layouts, head, layout);
            }
            let offset = *offsets.get(index).ok_or("missing C field offset")?;
            if offset % u64::from(child.align) != 0 {
                return Err("C aggregate ABI field is not naturally aligned");
            }
            if layout.size <= 16 {
                let class = if matches!(kind,Kind::Primitive(p) if p.is_float()) {
                    Class::Sse
                } else {
                    Class::Integer
                };
                let last = offset
                    .checked_add(child.size)
                    .and_then(|n| n.checked_sub(1))
                    .ok_or("C ABI field extent overflow")?;
                if last >= layout.size {
                    return Err("C ABI field exceeds object span");
                }
                for byte in offset / 8..=last / 8 {
                    let slot = classes
                        .get_mut(usize::try_from(byte).map_err(|_| "C ABI class index overflow")?)
                        .ok_or("C ABI field exceeds two eightbytes")?;
                    *slot = match (*slot, class) {
                        (Class::Integer, _) | (_, Class::Integer) => Class::Integer,
                        _ => class,
                    };
                }
            }
        }
        let classes = if layout.size > 16 {
            Classes::Memory
        } else {
            Classes::Eightbytes {
                first: classes[0],
                second: (layout.size > 8).then_some(classes[1]),
            }
        };
        Ok(Passing::Object(Object {
            bytes: layout.size,
            align: layout.align,
            classes,
        }))
    }
    fn composite(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        head: TypeId,
        layout: super::layout::Layout,
    ) -> Result<Passing, &'static str> {
        if self.composites.is_none() {
            self.retain(size_of::<composites::Cache>() + 16)?;
        }
        let classes = self.composites.get_or_insert_default().demand(
            types,
            definitions,
            layouts,
            head,
            &mut self.steps,
            &mut self.retained_bytes,
        )?;
        Ok(Passing::Object(Object {
            bytes: layout.size,
            align: layout.align,
            classes,
        }))
    }
}
fn scalar(kind: Kind) -> bool {
    matches!(kind,Kind::Primitive(p) if p!=types::Primitive::Str)
        || matches!(
            kind,
            Kind::Reference { .. } | Kind::Pointer { .. } | Kind::Function { .. }
        )
}

fn account_work(steps: &mut usize, work: usize) -> Result<(), &'static str> {
    *steps = steps
        .checked_add(work)
        .filter(|&n| n <= 4_000_000)
        .ok_or("C ABI classification work exceeds budget")?;
    Ok(())
}
fn storage(retained: &mut usize, bytes: usize) -> Result<(), &'static str> {
    *retained = retained
        .checked_add(bytes)
        .filter(|&n| n <= 64_000_000)
        .ok_or("C ABI classification exceeds retained budget")?;
    Ok(())
}
