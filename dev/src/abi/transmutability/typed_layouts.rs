//! Translate completed physical facts once; nominal identity never grants a source proof.
use super::{Budget, ByteRange, Language, LayoutId, Reference, budget::push};
use crate::{
    abi::layout::{Layout, Layouts, Shape},
    support::{hash::HashMap, retained::Growth},
    types::{Kind, Primitive, Store, TypeId, aggregate::Definitions},
};
mod composition;
#[cfg(test)]
#[path = "../../../tests/invariants/physical_languages.rs"]
mod tests;

pub(super) struct Describer<'a> {
    types: &'a Store,
    definitions: &'a Definitions,
    layouts: &'a Layouts,
    language: Language,
    ready: HashMap<TypeId, LayoutId>,
}

/// Consume actual completed ABI layouts, sharing repeated types across both roots.
/// Referent layouts must also be complete; references retain opaque recursive obligations.
/// # Errors
/// Rejects incomplete/unsupported layouts, invalid facts and exhausted query payments.
pub fn describe(
    types: &Store,
    definitions: &Definitions,
    layouts: &Layouts,
    roots: [TypeId; 2],
    budget: &mut Budget<'_>,
) -> Result<(Language, [LayoutId; 2]), &'static str> {
    let mut describer = Describer {
        types,
        definitions,
        layouts,
        language: Language::default(),
        ready: HashMap::default(),
    };
    let source = describer.demand(roots[0], budget)?;
    let destination = describer.demand(roots[1], budget)?;
    Ok((describer.language, [source, destination]))
}
impl Describer<'_> {
    fn layout(&self, ty: TypeId) -> Result<Layout, &'static str> {
        self.layouts
            .get(ty)
            .copied()
            .ok_or("transmutability requires a completed physical layout")
    }
    fn child(&self, ty: TypeId) -> Result<LayoutId, &'static str> {
        self.ready
            .get(&ty)
            .copied()
            .ok_or("missing typed layout language")
    }
    fn demand(&mut self, root: TypeId, budget: &mut Budget<'_>) -> Result<LayoutId, &'static str> {
        let mut stack = Vec::new();
        push(&mut stack, (root, false), budget)?;
        while let Some((ty, finish)) = stack.pop() {
            budget.charge(1, 0)?;
            if self.ready.contains_key(&ty) {
                continue;
            }
            let layout = self.layout(ty)?;
            let kind = self.types.kind(ty).ok_or("missing typed layout kind")?;
            if !finish {
                push(&mut stack, (ty, true), budget)?;
                match kind {
                    Kind::Tuple(_) | Kind::Adt(..) => {
                        for variant in self.layouts.variants(&layout) {
                            for &field in self
                                .types
                                .list(variant.fields)
                                .ok_or("missing physical fields")?
                            {
                                push(&mut stack, (field, false), budget)?;
                            }
                        }
                    }
                    Kind::Array { element, .. } => push(&mut stack, (element, false), budget)?,
                    _ => {}
                }
                continue;
            }
            let value = self.construct(ty, kind, layout, budget)?;
            let growth =
                Growth::map::<TypeId, LayoutId>(self.ready.len(), self.ready.capacity(), 1);
            budget.charge(growth.work, growth.bytes)?;
            self.ready
                .try_reserve(1)
                .map_err(|_| "typed layout index allocation failed")?;
            self.ready.insert(ty, value);
        }
        self.child(root)
    }
    fn construct(
        &mut self,
        ty: TypeId,
        kind: Kind,
        layout: Layout,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        match kind {
            Kind::Primitive(Primitive::Bool) => self.language.byte(ByteRange::BOOL, budget),
            Kind::Primitive(Primitive::Char) => self.character(budget),
            Kind::Primitive(primitive) if primitive.integer().is_some() || primitive.is_float() => {
                self.bytes(ByteRange::INITIALIZED, layout.size, budget)
            }
            Kind::Array { element, length } => {
                let Shape::Array {
                    element: actual,
                    stride,
                    length: count,
                } = layout.shape
                else {
                    return Err("array lost its physical layout shape");
                };
                if actual != element || count != length || stride != self.layout(element)?.size {
                    return Err("array language disagrees with its physical element stride");
                }
                self.language.repeat(self.child(element)?, count, budget)
            }
            Kind::Tuple(_) | Kind::Adt(..) => {
                let body = self.aggregate(ty, layout, budget)?;
                if matches!(kind, Kind::Adt(..)) {
                    self.language.nominal(body, budget)
                } else {
                    Ok(body)
                }
            }
            Kind::Reference {
                mutable,
                region,
                referent,
            } => {
                if layout.size != 8 || !matches!(layout.shape, Shape::Scalar) {
                    return Err("transmutability requires a supported thin reference layout");
                }
                let referent_layout = self.layout(referent)?;
                self.language.reference(
                    Reference {
                        mutable,
                        region,
                        referent,
                        size: referent_layout.size,
                        align: referent_layout.align,
                    },
                    budget,
                )
            }
            _ => Err("transmutability does not support this type's layout language"),
        }
    }
    fn bytes(
        &mut self,
        range: ByteRange,
        count: u64,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let byte = self.language.byte(range, budget)?;
        self.language.repeat(byte, count, budget)
    }
    fn character(&mut self, budget: &mut Budget<'_>) -> Result<LayoutId, &'static str> {
        let mut branches = Vec::new();
        for ranges in [
            [(0, 256), (0, 216), (0, 1), (0, 1)],
            [(0, 256), (224, 256), (0, 1), (0, 1)],
            [(0, 256), (0, 256), (1, 17), (0, 1)],
        ] {
            let mut node = self.language.epsilon(budget)?;
            for (start, end) in ranges {
                let byte = self.language.byte(ByteRange::new(start, end)?, budget)?;
                node = self.language.sequence(node, byte, budget)?;
            }
            push(&mut branches, node, budget)?;
        }
        self.language.alternatives(branches, budget)
    }
}
