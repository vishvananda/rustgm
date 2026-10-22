//! Unsized place alignment and field offsets never pretend to be a value layout.
use super::layout::{Layouts, MAX_SIZE, Range, padded, validate_repr};
use crate::support::hash::HashMap;
use crate::types::{
    Kind, ListId, Primitive, Store, TypeId,
    aggregate::{self, Definitions, Repr, Representation},
};

mod alignment;
pub use alignment::{Alignment, Offset};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Fields {
    pub types: ListId,
    offsets: Range,
    tail_alignment: Option<Alignment>,
}
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub align: u32,
    pub dynamic_alignment: Option<Alignment>,
    pub contains_explicit_align: bool,
    pub fields: Option<Fields>,
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Complete(Result<Id, &'static str>),
}
enum Step {
    Leaf(Layout),
    Tail(TypeId),
}

/// Definitions and normalized field instances must be stable for this cache.
/// A tail has no fixed size; only prefix fields demand ordinary value layouts.
#[derive(Default)]
pub struct Queries {
    states: HashMap<TypeId, State>,
    records: Vec<Layout>,
    offsets: Vec<u64>,
    scratch: Vec<u64>,
    path: Vec<TypeId>,
    pub steps: usize,
}
impl Queries {
    #[must_use]
    pub fn at(&self, id: Id) -> Option<&Layout> {
        self.records.get(id.0 as usize)
    }
    #[must_use]
    pub fn offsets(&self, fields: Fields) -> impl ExactSizeIterator<Item = Offset> + '_ {
        let count = fields.offsets.end - fields.offsets.start;
        (0..count).map(move |index| self.offset(fields, index))
    }
    fn offset(&self, fields: Fields, index: u32) -> Offset {
        let prefix = self.offsets[(fields.offsets.start + index) as usize];
        match fields.tail_alignment {
            Some(alignment) if index + 1 == fields.offsets.end - fields.offsets.start => {
                Offset::Aligned { prefix, alignment }
            }
            _ => Offset::Fixed(prefix),
        }
    }
    #[must_use]
    pub fn field_offset(&self, fields: Fields, index: u32) -> Option<Offset> {
        (index < fields.offsets.end - fields.offsets.start).then(|| self.offset(fields, index))
    }
    /// # Errors
    /// Rejects sized roots, unprepared fields, invalid representations, cycles,
    /// target-size overflow and exhausted work. Both outcomes are memoized.
    pub fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        root: TypeId,
    ) -> Result<Id, &'static str> {
        if let Some(State::Complete(result)) = self.states.get(&root) {
            return *result;
        }
        let mut result = self.follow(types, definitions, layouts, root);
        while let Some(ty) = self.path.pop() {
            result = result.and_then(|tail| self.aggregate(types, definitions, layouts, ty, tail));
            self.states.insert(ty, State::Complete(result));
        }
        self.states.insert(root, State::Complete(result));
        result
    }
    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.path.len()) > 1_000_000 {
            return Err("unsized prefix layout work budget exceeded");
        }
        Ok(())
    }
    fn save(&mut self, layout: Layout) -> Result<Id, &'static str> {
        let id = u32::try_from(self.records.len()).map_err(|_| "too many unsized layouts")?;
        self.records.push(layout);
        Ok(Id(id))
    }
    fn follow(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        mut ty: TypeId,
    ) -> Result<Id, &'static str> {
        loop {
            self.charge(1)?;
            match self.states.get(&ty) {
                Some(State::Complete(result)) => return *result,
                Some(State::Active) => return Err("recursive unsized storage tail"),
                None => {}
            }
            match inspect(types, definitions, layouts, ty) {
                Ok(Step::Leaf(leaf)) => {
                    let result = self.save(leaf);
                    self.states.insert(ty, State::Complete(result));
                    return result;
                }
                Ok(Step::Tail(tail)) => {
                    self.states.insert(ty, State::Active);
                    self.path.push(ty);
                    ty = tail;
                }
                Err(error) => {
                    self.states.insert(ty, State::Complete(Err(error)));
                    return Err(error);
                }
            }
        }
    }
    fn aggregate(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        ty: TypeId,
        tail: Id,
    ) -> Result<Id, &'static str> {
        let (list, repr) = fields(types, definitions, ty)?;
        let fields = types.list(list).ok_or("invalid unsized fields")?;
        self.charge(fields.len())?;
        let tail = *self.at(tail).ok_or("missing unsized tail layout")?;
        let mut align = repr.align.unwrap_or(1);
        let mut size = 0_u64;
        let mut explicit = repr.align.is_some() || tail.contains_explicit_align;
        self.scratch.clear();
        for &field in fields.iter().take(fields.len() - 1) {
            let id = layouts
                .demand(types, definitions, field)
                .map_err(|e| e.message)?;
            let field = layouts.at(id).ok_or("missing fixed prefix layout")?;
            explicit |= field.contains_explicit_align;
            if repr.representation == Representation::Transparent {
                if field.size != 0 || field.align != 1 {
                    return Err("transparent unsized aggregate has a nontrivial prefix field");
                }
                self.scratch.push(0);
            } else {
                let field_align = repr.packed.map_or(field.align, |p| p.min(field.align));
                align = align.max(field_align);
                size = padded(size, field_align)?;
                self.scratch.push(size);
                size = size
                    .checked_add(field.size)
                    .filter(|n| *n <= MAX_SIZE)
                    .ok_or("unsized prefix exceeds target object limit")?;
            }
        }
        if !definitions.packing_checked() && repr.packed.is_some() && explicit {
            return Err("packed aggregate transitively contains an explicitly aligned type");
        }
        let tail_alignment = tail
            .dynamic_alignment
            .unwrap_or_else(|| Alignment::fixed(tail.align))
            .cap(repr.packed);
        let alignment = tail_alignment.raise(align);
        let dynamic_tail = (tail_alignment.is_dynamic() && size != 0).then_some(tail_alignment);
        self.scratch.push(if dynamic_tail.is_some() {
            size
        } else {
            padded(size, tail_alignment.minimum)?
        });
        let start = u32::try_from(self.offsets.len()).map_err(|_| "too many tail field offsets")?;
        self.offsets.extend_from_slice(&self.scratch);
        let end = u32::try_from(self.offsets.len()).map_err(|_| "too many tail field offsets")?;
        self.save(Layout {
            align: alignment.minimum,
            dynamic_alignment: alignment.is_dynamic().then_some(alignment),
            contains_explicit_align: explicit,
            fields: Some(Fields {
                types: list,
                offsets: Range { start, end },
                tail_alignment: dynamic_tail,
            }),
        })
    }
}

fn inspect(
    types: &Store,
    definitions: &Definitions,
    layouts: &mut Layouts,
    ty: TypeId,
) -> Result<Step, &'static str> {
    match types.kind(ty).ok_or("invalid unsized layout type")? {
        Kind::Primitive(Primitive::Str) => Ok(Step::Leaf(Layout {
            align: 1,
            dynamic_alignment: None,
            contains_explicit_align: false,
            fields: None,
        })),
        Kind::Dynamic { .. } => Ok(Step::Leaf(Layout {
            align: 1,
            dynamic_alignment: Some(Alignment::OBJECT),
            contains_explicit_align: false,
            fields: None,
        })),
        Kind::Slice(element) => {
            let id = layouts
                .demand(types, definitions, element)
                .map_err(|e| e.message)?;
            let element = layouts.at(id).ok_or("missing slice element layout")?;
            Ok(Step::Leaf(Layout {
                align: element.align,
                dynamic_alignment: None,
                contains_explicit_align: element.contains_explicit_align,
                fields: None,
            }))
        }
        Kind::Adt(..) | Kind::Tuple(_) => {
            let (fields, _) = fields(types, definitions, ty)?;
            types
                .list(fields)
                .and_then(|list| list.last())
                .copied()
                .map(Step::Tail)
                .ok_or("unsized aggregate has no storage tail")
        }
        _ => Err("unsized layout requires a concrete storage tail"),
    }
}

fn fields(
    types: &Store,
    definitions: &Definitions,
    ty: TypeId,
) -> Result<(ListId, Repr), &'static str> {
    match types.kind(ty) {
        Some(Kind::Tuple(list)) => Ok((list, Repr::default())),
        Some(Kind::Adt(..)) => {
            let shape = definitions.shape(types, ty)?;
            if shape.definition.kind != aggregate::Kind::Struct {
                return Err("an enum cannot have an unsized storage tail");
            }
            validate_repr(shape.definition, shape.variants().len())?;
            Ok((
                shape
                    .variant(0)
                    .ok_or("missing unsized struct fields")?
                    .fields,
                shape.definition.repr,
            ))
        }
        _ => Err("unsized prefix requires a struct or tuple"),
    }
}
