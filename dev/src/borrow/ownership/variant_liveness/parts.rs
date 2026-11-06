//! Paid structural coordinates for cleanup tags, never semantic field proof.
use crate::{
    support::{hash::HashMap, retained::Growth},
    types::{
        ItemId, Kind, Store, TypeId,
        aggregate::{Definitions, FieldDestruction, Kind as NominalKind},
    },
};
use std::cell::Cell;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Id(pub u32);
#[derive(Clone, Copy, Default)]
pub(super) struct Range {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy)]
pub(super) enum Form {
    Leaf,
    Tuple,
    Nominal { enumeration: bool },
    Array,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Need {
    None,
    Whole,
    OwnDrop,
}
pub(super) struct Part {
    pub ty: TypeId,
    pub form: Form,
    pub variants: Range,
    pub tagged: bool,
    pub interior: bool,
    pub need: Need,
    pub relevant: bool,
}
pub(super) struct Variant {
    pub fields: Range,
}
pub(super) struct Field {
    pub ty: TypeId,
    pub part: Option<Id>,
}
#[derive(Clone, Copy)]
enum State {
    Active(Id),
    Ready(Id),
}
enum Work {
    Enter(TypeId),
    Finish(Id),
}
pub(super) struct Parts {
    pub entries: Vec<Part>,
    pub variants: Vec<Variant>,
    pub fields: Vec<Field>,
    index: HashMap<TypeId, State>,
    work: Vec<Work>,
    pub steps: Cell<usize>,
    bytes: Cell<usize>,
    limit: usize,
}
impl Parts {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            variants: Vec::new(),
            fields: Vec::new(),
            index: HashMap::default(),
            work: Vec::new(),
            steps: Cell::new(0),
            bytes: Cell::new(0),
            limit,
        }
    }
    pub(super) fn charge(&self, amount: usize) -> Result<(), &'static str> {
        self.steps.set(
            self.steps
                .get()
                .checked_add(amount)
                .filter(|&n| n <= 32_000_000)
                .ok_or("cleanup tag descriptor work budget exceeded")?,
        );
        Ok(())
    }
    pub(super) fn pay(&self, growth: Growth) -> Result<(), &'static str> {
        self.charge(growth.work)?;
        self.bytes.set(
            self.bytes
                .get()
                .checked_add(growth.bytes)
                .filter(|&n| n <= self.limit)
                .ok_or("cleanup tag descriptors exceed storage budget")?,
        );
        Ok(())
    }
    fn push(&mut self, work: Work) -> Result<(), &'static str> {
        self.pay(Growth::vector::<Work>(
            self.work.len(),
            self.work.capacity(),
            1,
        ))?;
        self.work
            .try_reserve(1)
            .map_err(|_| "cleanup tag work allocation failed")?;
        self.work.push(work);
        Ok(())
    }
    pub(super) fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        custom: &dyn Fn(ItemId) -> bool,
    ) -> Result<Id, &'static str> {
        self.push(Work::Enter(ty))?;
        while let Some(work) = self.work.pop() {
            self.charge(1)?;
            match work {
                Work::Enter(ty) => self.enter(types, definitions, ty, custom)?,
                Work::Finish(id) => self.finish(id)?,
            }
        }
        match self.index.get(&ty) {
            Some(State::Ready(id)) => Ok(*id),
            _ => Err("cleanup tag descriptor did not complete"),
        }
    }
    fn enter(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        custom: &dyn Fn(ItemId) -> bool,
    ) -> Result<(), &'static str> {
        match self.index.get(&ty) {
            Some(State::Ready(_)) => return Ok(()),
            Some(State::Active(_)) => return Err("recursive owned cleanup tag shape"),
            None => {}
        }
        self.pay(Growth::vector::<Part>(
            self.entries.len(),
            self.entries.capacity(),
            1,
        ))?;
        self.pay(Growth::map::<TypeId, State>(
            self.index.len(),
            self.index.capacity(),
            1,
        ))?;
        self.entries
            .try_reserve(1)
            .map_err(|_| "cleanup tag part allocation failed")?;
        self.index
            .try_reserve(1)
            .map_err(|_| "cleanup tag index allocation failed")?;
        let id = Id(compact(self.entries.len())?);
        let start = compact(self.variants.len())?;
        let form = self.describe(types, definitions, ty)?;
        let end = compact(self.variants.len())?;
        let interior = matches!(types.kind(ty),Some(Kind::Adt(item,_))
            if definitions.shared_mutation(item) == Some(crate::types::aggregate::SharedMutation::Interior));
        let need = match types
            .kind(ty)
            .ok_or("cleanup qualification type is missing")?
        {
            Kind::Adt(item, _)
                if definitions.destruction(item) == Some(FieldDestruction::Never) =>
            {
                Need::None
            }
            Kind::Adt(item, _) if custom(item) => Need::OwnDrop,
            Kind::Parameter(_)
            | Kind::Projection(_)
            | Kind::Opaque(..)
            | Kind::Future { .. }
            | Kind::Dynamic { .. }
            | Kind::Foreign(_) => Need::Whole,
            _ => Need::None,
        };
        self.entries.push(Part {
            ty,
            form,
            variants: Range { start, end },
            tagged: false,
            interior,
            need,
            relevant: need != Need::None,
        });
        self.index.insert(ty, State::Active(id));
        self.push(Work::Finish(id))?;
        for index in start..end {
            let range = self.variants[index as usize].fields;
            for field in range.start..range.end {
                self.push(Work::Enter(self.fields[field as usize].ty))?;
            }
        }
        Ok(())
    }
    fn describe(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<Form, &'static str> {
        match types.kind(ty).ok_or("cleanup tag type is missing")? {
            Kind::Tuple(fields) => {
                self.list(types, fields)?;
                Ok(Form::Tuple)
            }
            Kind::Closure { environment, .. } => {
                let Some(Kind::Tuple(fields)) = types.kind(environment) else {
                    return Err("cleanup tag closure has no tuple environment");
                };
                self.list(types, fields)?;
                Ok(Form::Tuple)
            }
            Kind::Adt(item, _) => {
                let definition = definitions
                    .get(item)
                    .ok_or("cleanup tag nominal is pending")?;
                if definition.kind == NominalKind::Union
                    || definitions.destruction(item) != Some(FieldDestruction::Fields)
                {
                    return Ok(Form::Leaf);
                }
                for variant in definitions.shape(types, ty)?.variants() {
                    self.list(types, variant.fields)?;
                }
                Ok(Form::Nominal {
                    enumeration: definition.kind == NominalKind::Enum,
                })
            }
            Kind::Array { length: 0, .. } => Ok(Form::Leaf),
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => {
                self.variant(&[element])?;
                Ok(Form::Array)
            }
            _ => Ok(Form::Leaf),
        }
    }
    fn list(&mut self, types: &Store, fields: crate::types::ListId) -> Result<(), &'static str> {
        self.variant(
            types
                .list(fields)
                .ok_or("cleanup tag field list is missing")?,
        )
    }
    fn variant(&mut self, fields: &[TypeId]) -> Result<(), &'static str> {
        self.charge(fields.len().saturating_add(1))?;
        self.pay(Growth::vector::<Variant>(
            self.variants.len(),
            self.variants.capacity(),
            1,
        ))?;
        self.pay(Growth::vector::<Field>(
            self.fields.len(),
            self.fields.capacity(),
            fields.len(),
        ))?;
        self.variants
            .try_reserve(1)
            .map_err(|_| "cleanup tag variant allocation failed")?;
        self.fields
            .try_reserve(fields.len())
            .map_err(|_| "cleanup tag field allocation failed")?;
        let start = compact(self.fields.len())?;
        for &ty in fields {
            self.fields.push(Field { ty, part: None });
        }
        self.variants.push(Variant {
            fields: Range {
                start,
                end: compact(self.fields.len())?,
            },
        });
        Ok(())
    }
    fn finish(&mut self, id: Id) -> Result<(), &'static str> {
        let part = self
            .entries
            .get(id.0 as usize)
            .ok_or("cleanup tag part disappeared")?;
        let ty = part.ty;
        if !matches!(self.index.get(&ty),Some(State::Active(actual)) if *actual == id) {
            return Err("cleanup tag completion changed its structural owner");
        }
        let mut tagged = matches!(part.form, Form::Nominal { enumeration: true });
        let mut interior = part.interior;
        let mut relevant = part.relevant || tagged;
        let variants = part.variants;
        for variant in variants.start..variants.end {
            let fields = self.variants[variant as usize].fields;
            self.charge((fields.end - fields.start) as usize)?;
            for index in fields.start..fields.end {
                let field = &mut self.fields[index as usize];
                let Some(State::Ready(child)) = self.index.get(&field.ty) else {
                    return Err("cleanup tag child descriptor is pending");
                };
                field.part = Some(*child);
                tagged |= self.entries[child.0 as usize].tagged;
                interior |= self.entries[child.0 as usize].interior;
                relevant |= self.entries[child.0 as usize].relevant;
            }
        }
        self.entries[id.0 as usize].tagged = tagged;
        self.entries[id.0 as usize].interior = interior;
        self.entries[id.0 as usize].relevant = relevant;
        self.index.insert(ty, State::Ready(id));
        Ok(())
    }
}
pub(super) fn compact(value: usize) -> Result<u32, &'static str> {
    u32::try_from(value).map_err(|_| "cleanup tag descriptor identity limit exceeded")
}
