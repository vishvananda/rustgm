//! Callable demands come from executable uses and typed constant storage graphs.
use crate::{
    abi::layout::Layouts,
    body::{
        self,
        ir::{Body, Rvalue},
    },
    sema::{
        model::{Cast, Declarations},
        values::{Constant, LocationId, Origin, Reader, View},
    },
    types::{Kind, TypeId, instance::Instance},
};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct Dependencies {
    values: HashSet<(TypeId, Constant)>,
    locations: HashSet<LocationId>,
    steps: usize,
}
enum Work {
    Value(TypeId, Constant),
    Location(LocationId),
}
impl Dependencies {
    pub(super) fn collect(
        &mut self,
        facts: &mut Declarations,
        body: &Body,
    ) -> Result<Vec<Instance>, &'static str> {
        let mut layouts = std::mem::take(&mut facts.layouts);
        let result = self.run(facts, &mut layouts, body);
        facts.layouts = layouts;
        result
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err("callable dependency work budget exceeded");
        }
        Ok(())
    }
    fn run(
        &mut self,
        facts: &Declarations,
        layouts: &mut Layouts,
        body: &Body,
    ) -> Result<Vec<Instance>, &'static str> {
        let reader = Reader::new(facts);
        let mut result = Vec::new();
        let mut work = Vec::new();
        self.charge(body.values.len())?;
        for value in &body.values {
            let operand = match value.kind {
                Rvalue::Reify(id)
                | Rvalue::Cast {
                    kind: Cast::FunctionPointer,
                    operand: id,
                }
                | Rvalue::Call { callee: id, .. } => Some(id),
                _ => None,
            };
            if let Some(id) = operand {
                let ty = body
                    .values
                    .get(id.0 as usize)
                    .ok_or("invalid callable operand")?
                    .ty;
                if matches!(facts.types.kind(ty), Some(Kind::FunctionItem { .. })) {
                    result.push(body.callable_target(Instance::new(&facts.types, ty)?));
                }
            }
        }
        for (id, _) in body::constants::references(body) {
            self.charge(1)?;
            let value = reader
                .definition(id)
                .ok_or("missing completed callable constant")?;
            let ty = facts
                .definitions
                .get(id.0 as usize)
                .copied()
                .flatten()
                .ok_or("missing callable constant type")?;
            work.push(Work::Value(ty, value));
        }
        for value in &body.values {
            if let Rvalue::AssociatedConstant(instance) = value.kind {
                self.charge(1)?;
                work.push(Work::Value(
                    value.ty,
                    reader
                        .associated(instance)
                        .ok_or("missing completed associated constant")?,
                ));
            }
        }
        if let Some(promotions) = &body.promotions {
            for (id, value) in promotions.entries() {
                self.charge(1)?;
                let ty = body
                    .values
                    .get(id.0 as usize)
                    .ok_or("invalid promoted callable value")?
                    .ty;
                work.push(Work::Value(ty, value));
            }
        }
        self.follow(facts, layouts, work, &mut result)?;
        Ok(result)
    }
    fn follow(
        &mut self,
        facts: &Declarations,
        layouts: &mut Layouts,
        mut work: Vec<Work>,
        result: &mut Vec<Instance>,
    ) -> Result<(), &'static str> {
        let reader = Reader::new(facts);
        while let Some(next) = work.pop() {
            self.charge(1)?;
            match next {
                Work::Location(id) => {
                    if !self.locations.insert(id) {
                        continue;
                    }
                    let location = reader.location(id)?;
                    match location.origin {
                        Origin::Projection { base, .. } => work.push(Work::Location(base)),
                        Origin::Allocation => work.push(Work::Value(
                            location.ty,
                            location.value.ok_or("missing allocation value")?,
                        )),
                        Origin::Static(id) => work.push(Work::Value(
                            location.ty,
                            reader.definition(id).ok_or("missing static value")?,
                        )),
                    }
                }
                Work::Value(ty, value) => {
                    if !self.values.insert((ty, value)) {
                        continue;
                    }
                    let layout = layouts
                        .demand(&facts.types, &facts.aggregates, ty)
                        .map_err(|e| e.message)?;
                    if layouts.at(layout).ok_or("missing constant layout")?.size == 0 {
                        continue;
                    }
                    match reader.view(value)? {
                        View::Function(instance) => result.push(instance),
                        View::Reference(id) => work.push(Work::Location(id)),
                        View::Repeat { value, length } if length != 0 => {
                            let Some(Kind::Array { element, .. }) = facts.types.kind(ty) else {
                                return Err("repeated constant is not an array");
                            };
                            work.push(Work::Value(element, value));
                        }
                        View::Product { length, variant } => {
                            self.charge(
                                usize::try_from(length)
                                    .map_err(|_| "constant product exceeds host size")?,
                            )?;
                            for index in (0..length).rev() {
                                let child = field(facts, ty, variant, index)?;
                                work.push(Work::Value(child, reader.field(value, index)?));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
fn field(
    facts: &Declarations,
    ty: TypeId,
    variant: Option<crate::sema::members::VariantId>,
    index: u64,
) -> Result<TypeId, &'static str> {
    let index = usize::try_from(index).map_err(|_| "constant field exceeds host size")?;
    match facts.types.kind(ty) {
        Some(Kind::Array { element, .. }) => Ok(element),
        Some(Kind::Tuple(list)) => facts
            .types
            .list(list)
            .and_then(|l| l.get(index))
            .copied()
            .ok_or("invalid tuple constant field"),
        Some(Kind::Adt(item, _)) => {
            let ordinal = if let Some(id) = variant {
                let variant = facts
                    .members
                    .variants
                    .get(id.0 as usize)
                    .ok_or("invalid constant variant")?;
                if variant.owner != item {
                    return Err("constant variant belongs to another type");
                }
                variant.ordinal as usize
            } else {
                0
            };
            facts
                .aggregates
                .shape(&facts.types, ty)?
                .variant(ordinal)
                .and_then(|variant| facts.types.list(variant.fields))
                .and_then(|l| l.get(index))
                .copied()
                .ok_or("invalid nominal constant field")
        }
        _ => Err("constant product has no field types"),
    }
}
