//! Iterative x86-64 layout queries; nominal identity is separate from field dependencies.
use crate::types::{
    Kind, ListId, Primitive, Store, TypeId,
    aggregate::{self, Definitions, Repr, Representation},
};

const MAX_SIZE: u64 = i64::MAX as u64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutId(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct Tag {
    pub ty: Primitive,
    pub offset: u64,
}
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Scalar,
    Array {
        element: TypeId,
        stride: u64,
        length: u64,
    },
    Aggregate {
        variants: Range,
        tag: Option<Tag>,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub size: u64,
    pub align: u32,
    pub inhabited: bool,
    pub contains_explicit_align: bool,
    /// By-value references remain visible when an aggregate is represented as bytes.
    pub contains_reference: bool,
    pub shape: Shape,
}
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    pub fields: ListId,
    pub offsets: Range,
    pub discriminant: u128,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub ty: TypeId,
    pub message: &'static str,
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "type {}: {}", self.ty.0, self.message)
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Unseen,
    Active,
    Ready(LayoutId),
    Failed(Error),
}

/// Cached queries belong to one canonical store and stable, completed definitions.
/// Appending canonical types is allowed; changing an existing definition is not.
#[derive(Default)]
pub struct Layouts {
    states: Vec<State>,
    records: Vec<Layout>,
    variants: Vec<Variant>,
    offsets: Vec<u64>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl Layouts {
    /// Required reference alignment, including referents without a fixed size.
    /// Pointer storage alignment does not depend on its referent's metadata.
    /// # Errors
    /// Preserves layout errors for sized referents and slice elements.
    pub fn reference_align(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<u32, Error> {
        let ty = match types.kind(ty) {
            Some(Kind::Primitive(Primitive::Str)) => return Ok(1),
            Some(Kind::Reference { .. } | Kind::Pointer { .. }) => return Ok(8),
            Some(Kind::Slice(element)) => element,
            _ => ty,
        };
        let id = self.demand(types, definitions, ty)?;
        Ok(self.records[id.0 as usize].align)
    }
    #[must_use]
    pub fn get(&self, ty: TypeId) -> Option<&Layout> {
        let State::Ready(id) = self.states.get(ty.0 as usize)? else {
            return None;
        };
        self.records.get(id.0 as usize)
    }
    #[must_use]
    pub fn at(&self, id: LayoutId) -> Option<&Layout> {
        self.records.get(id.0 as usize)
    }
    #[must_use]
    pub fn variants(&self, layout: &Layout) -> &[Variant] {
        if let Shape::Aggregate { variants, .. } = layout.shape {
            &self.variants[variants.start as usize..variants.end as usize]
        } else {
            &[]
        }
    }
    #[must_use]
    pub fn offsets(&self, variant: &Variant) -> &[u64] {
        &self.offsets[variant.offsets.start as usize..variant.offsets.end as usize]
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Compute each demanded layout once, traversing only by-value dependencies.
    ///
    /// # Errors
    /// Rejects cycles, unsized storage, invalid representations and target-size overflow.
    pub fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<LayoutId, Error> {
        if self.states.len() < types.len() {
            self.states.resize(types.len(), State::Unseen);
        }
        if let Some(State::Ready(id)) = self.states.get(ty.0 as usize) {
            return Ok(*id);
        }
        if let Some(State::Failed(error)) = self.states.get(ty.0 as usize) {
            return Err(*error);
        }
        self.work.push((ty, false));
        while let Some((next, finish)) = self.work.pop() {
            self.steps += 1;
            if let Err(message) = self.visit(types, definitions, next, finish) {
                let error = Error { ty: next, message };
                // Completed dependencies remain reusable. Only the failed query's
                // unfinished records are rolled back by its construction boundary.
                for state in &mut self.states {
                    if matches!(state, State::Active) {
                        *state = State::Failed(error);
                    }
                }
                if let Some(state) = self.states.get_mut(ty.0 as usize) {
                    *state = State::Failed(error);
                }
                self.work.clear();
                return Err(error);
            }
        }
        match self.states.get(ty.0 as usize) {
            Some(State::Ready(id)) => Ok(*id),
            _ => Err(Error {
                ty,
                message: "layout query did not complete",
            }),
        }
    }
    fn visit(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        finish: bool,
    ) -> Result<(), &'static str> {
        let state = *self
            .states
            .get(ty.0 as usize)
            .ok_or("invalid layout type")?;
        match state {
            State::Ready(_) => return Ok(()),
            State::Failed(error) => return Err(error.message),
            State::Active if !finish => return Err("recursive type has infinite size"),
            _ => {}
        }
        let kind = types.kind(ty).ok_or("invalid layout type")?;
        if finish {
            let variant_mark = self.variants.len();
            let offset_mark = self.offsets.len();
            let result = self.construct(types, definitions, kind);
            let layout = match result {
                Ok(layout) => layout,
                Err(error) => {
                    self.variants.truncate(variant_mark);
                    self.offsets.truncate(offset_mark);
                    return Err(error);
                }
            };
            let id = LayoutId(index(self.records.len())?);
            self.records.push(layout);
            self.states[ty.0 as usize] = State::Ready(id);
            return Ok(());
        }
        self.states[ty.0 as usize] = State::Active;
        self.work.push((ty, true));
        match kind {
            Kind::Tuple(fields) => self.dependencies(types, fields)?,
            Kind::Array { element, .. } => self.work.push((element, false)),
            Kind::Adt(item, _) => {
                let definition = definitions
                    .get(item)
                    .ok_or("missing completed aggregate definition")?;
                for variant in definitions.variants(definition).iter().rev() {
                    self.dependencies(types, variant.fields)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn dependencies(&mut self, types: &Store, fields: ListId) -> Result<(), &'static str> {
        self.work.extend(
            types
                .list(fields)
                .ok_or("invalid layout field list")?
                .iter()
                .rev()
                .map(|ty| (*ty, false)),
        );
        Ok(())
    }
    fn construct(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        kind: Kind,
    ) -> Result<Layout, &'static str> {
        match kind {
            Kind::Never => Ok(Layout {
                size: 0,
                align: 1,
                inhabited: false,
                contains_explicit_align: false,
                contains_reference: false,
                shape: Shape::Scalar,
            }),
            Kind::Primitive(p) => primitive(p),
            Kind::Function { .. } => Ok(scalar(8, 8)),
            Kind::FunctionItem { .. } => Ok(scalar(0, 1)),
            Kind::Pointer { pointee, .. }
            | Kind::Reference {
                referent: pointee, ..
            } => {
                if types.is_sized(pointee) != Some(true) {
                    return Err("unsized pointer metadata requires later layout support");
                }
                Ok(Layout {
                    contains_reference: matches!(kind, Kind::Reference { .. }),
                    ..scalar(8, 8)
                })
            }
            Kind::Slice(_) => Err("unsized slice has no fixed layout"),
            Kind::Array { element, length } => {
                let element_layout = self.get(element).ok_or("missing array element layout")?;
                let size = element_layout
                    .size
                    .checked_mul(length)
                    .filter(|n| *n <= MAX_SIZE)
                    .ok_or("array size exceeds target object limit")?;
                Ok(Layout {
                    size,
                    align: element_layout.align,
                    inhabited: length == 0 || element_layout.inhabited,
                    contains_explicit_align: element_layout.contains_explicit_align,
                    contains_reference: element_layout.contains_reference,
                    shape: Shape::Array {
                        element,
                        stride: element_layout.size,
                        length,
                    },
                })
            }
            Kind::Tuple(fields) => {
                let variants = Range {
                    start: index(self.variants.len())?,
                    end: index(self.variants.len() + 1)?,
                };
                let layout = self.fields(types, fields, Repr::default(), 0, 0)?;
                Ok(Layout {
                    shape: Shape::Aggregate {
                        variants,
                        tag: None,
                    },
                    ..layout
                })
            }
            Kind::Adt(item, _) => {
                let definition = definitions.get(item).ok_or("missing aggregate shape")?;
                self.aggregate(types, definition, definitions.variants(definition))
            }
        }
    }
    fn fields(
        &mut self,
        types: &Store,
        fields: ListId,
        repr: Repr,
        base: u64,
        discriminant: u128,
    ) -> Result<Layout, &'static str> {
        let start = index(self.offsets.len())?;
        let mut size = base;
        let mut align = repr.align.unwrap_or(1);
        let mut inhabited = true;
        let mut contains_explicit_align = repr.align.is_some();
        let mut contains_reference = false;
        let mut transparent = None;
        for ty in types.list(fields).ok_or("missing aggregate fields")? {
            let field = *self.get(*ty).ok_or("missing aggregate field layout")?;
            if repr.packed.is_some() && field.contains_explicit_align {
                return Err("packed aggregate transitively contains an explicitly aligned type");
            }
            contains_explicit_align |= field.contains_explicit_align;
            contains_reference |= field.contains_reference;
            inhabited &= field.inhabited;
            if repr.representation == Representation::Transparent {
                if field.size != 0 || field.align != 1 {
                    if transparent.is_some() {
                        return Err("transparent aggregate has more than one nontrivial field");
                    }
                    transparent = Some(field);
                }
                self.offsets.push(0);
            } else {
                let field_align = repr
                    .packed
                    .map_or(field.align, |pack| pack.min(field.align));
                align = align.max(field_align);
                size = padded(size, field_align)?;
                self.offsets.push(size);
                size = size
                    .checked_add(field.size)
                    .filter(|n| *n <= MAX_SIZE)
                    .ok_or("aggregate size exceeds target object limit")?;
            }
        }
        if repr.representation == Representation::Transparent {
            let principal = transparent.unwrap_or_else(|| scalar(0, 1));
            size = principal.size;
            align = principal.align;
        }
        let offsets = Range {
            start,
            end: index(self.offsets.len())?,
        };
        self.variants.push(Variant {
            fields,
            offsets,
            discriminant,
        });
        Ok(Layout {
            size: padded(size, align)?,
            align,
            inhabited,
            shape: Shape::Scalar,
            contains_explicit_align,
            contains_reference,
        })
    }
    fn aggregate(
        &mut self,
        types: &Store,
        definition: &aggregate::Definition,
        variants: &[aggregate::Variant],
    ) -> Result<Layout, &'static str> {
        let repr = definition.repr;
        validate_repr(definition, variants.len())?;
        let range = Range {
            start: index(self.variants.len())?,
            end: index(self.variants.len() + variants.len())?,
        };
        if definition.kind == aggregate::Kind::Struct
            || repr.representation == Representation::Transparent
        {
            let variant = variants.first().ok_or("aggregate has no field group")?;
            let layout = self.fields(types, variant.fields, repr, 0, variant.discriminant)?;
            return Ok(Layout {
                shape: Shape::Aggregate {
                    variants: range,
                    tag: None,
                },
                ..layout
            });
        }
        let no_tag = repr.representation == Representation::Rust
            && repr.tag.is_none()
            && variants.len() <= 1;
        let tag = (!no_tag).then_some(Tag {
            ty: enum_tag(repr, variants)?,
            offset: 0,
        });
        let tag_layout = tag.map_or(Ok(scalar(0, 1)), |tag| primitive(tag.ty))?;
        let inline_tag = repr.tag.is_some() && repr.representation != Representation::C;
        let mut size = 0;
        let mut align = repr.align.unwrap_or(1).max(tag_layout.align);
        let mut payload_align = 1;
        let mut inhabited = false;
        let mut contains_explicit_align = repr.align.is_some();
        let mut contains_reference = false;
        for variant in variants {
            let layout = self.fields(
                types,
                variant.fields,
                Repr::default(),
                if inline_tag { tag_layout.size } else { 0 },
                variant.discriminant,
            )?;
            size = size.max(layout.size);
            align = align.max(layout.align);
            payload_align = payload_align.max(layout.align);
            inhabited |= layout.inhabited;
            contains_explicit_align |= layout.contains_explicit_align;
            contains_reference |= layout.contains_reference;
        }
        if !inline_tag {
            let base = padded(tag_layout.size, payload_align)?;
            for variant in &self.variants[range.start as usize..range.end as usize] {
                for offset in
                    &mut self.offsets[variant.offsets.start as usize..variant.offsets.end as usize]
                {
                    *offset = offset
                        .checked_add(base)
                        .filter(|n| *n <= MAX_SIZE)
                        .ok_or("enum field offset exceeds target object limit")?;
                }
            }
            size = size
                .checked_add(base)
                .filter(|n| *n <= MAX_SIZE)
                .ok_or("enum size exceeds target object limit")?;
        }
        Ok(Layout {
            size: padded(size, align)?,
            align,
            inhabited,
            shape: Shape::Aggregate {
                variants: range,
                tag,
            },
            contains_explicit_align,
            contains_reference,
        })
    }
}
fn scalar(size: u64, align: u32) -> Layout {
    Layout {
        size,
        align,
        inhabited: true,
        contains_explicit_align: false,
        contains_reference: false,
        shape: Shape::Scalar,
    }
}
fn primitive(primitive: Primitive) -> Result<Layout, &'static str> {
    let bytes = match primitive {
        Primitive::Bool | Primitive::I8 | Primitive::U8 => 1,
        Primitive::I16 | Primitive::U16 => 2,
        Primitive::Char | Primitive::I32 | Primitive::U32 | Primitive::F32 => 4,
        Primitive::I64 | Primitive::U64 | Primitive::Isize | Primitive::Usize | Primitive::F64 => 8,
        Primitive::I128 | Primitive::U128 => 16,
        Primitive::Str => return Err("unsized str has no fixed layout"),
    };
    Ok(scalar(u64::from(bytes), bytes))
}
fn padded(size: u64, align: u32) -> Result<u64, &'static str> {
    let mask = u64::from(align) - 1;
    size.checked_add(mask)
        .map(|n| n & !mask)
        .filter(|n| *n <= MAX_SIZE)
        .ok_or("aligned size exceeds target object limit")
}
fn index(size: usize) -> Result<u32, &'static str> {
    u32::try_from(size).map_err(|_| "layout identity space exhausted")
}
fn validate_repr(definition: &aggregate::Definition, variants: usize) -> Result<(), &'static str> {
    let repr = definition.repr;
    for align in [repr.align, repr.packed].into_iter().flatten() {
        if !align.is_power_of_two() || align > (1 << 29) {
            return Err("invalid representation alignment");
        }
    }
    if repr.align.is_some() && repr.packed.is_some() {
        return Err("packed and align representations conflict");
    }
    if repr.representation == Representation::Transparent
        && (variants != 1 || repr.align.is_some() || repr.packed.is_some() || repr.tag.is_some())
    {
        return Err("transparent representation requires one variant and no other modifiers");
    }
    match definition.kind {
        aggregate::Kind::Struct if repr.tag.is_some() => {
            Err("integer representation requires an enum")
        }
        aggregate::Kind::Enum if repr.packed.is_some() => {
            Err("packed representation requires a struct")
        }
        aggregate::Kind::Enum
            if variants == 0
                && (repr.tag.is_some() || repr.representation != Representation::Rust) =>
        {
            Err("empty enum cannot have an explicit representation")
        }
        _ if repr.tag.is_some_and(|p| p.integer().is_none()) => {
            Err("enum tag must have an integer type")
        }
        _ => Ok(()),
    }
}

fn enum_tag(repr: Repr, variants: &[aggregate::Variant]) -> Result<Primitive, &'static str> {
    if let Some(tag) = repr.tag {
        return Ok(tag);
    }
    if repr.representation != Representation::C {
        return Ok(Primitive::I64);
    }
    let mut minimum = 0_i128;
    let mut maximum = 0_i128;
    for variant in variants {
        let bits = u64::try_from(variant.discriminant)
            .map_err(|_| "default enum discriminant exceeds isize representation")?;
        let value = i128::from(bits)
            - if bits >= (1_u64 << 63) {
                1_i128 << 64
            } else {
                0
            };
        minimum = minimum.min(value);
        maximum = maximum.max(value);
    }
    Ok(if minimum < 0 {
        if minimum >= i128::from(i32::MIN) && maximum <= i128::from(i32::MAX) {
            Primitive::I32
        } else {
            Primitive::I64
        }
    } else if maximum <= i128::from(u32::MAX) {
        Primitive::U32
    } else {
        Primitive::U64
    })
}
