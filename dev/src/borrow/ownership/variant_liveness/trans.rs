//! One checked statement index supplies both tag dataflow and replay transfers.
use super::{
    parts::{self, Form, Range},
    tags::{Coordinate, Located, Step, Tags},
};
use crate::{
    body::{
        drops::{MemberPosition, MemberQuery},
        ir::{Action, BlockId, Body, LocalKind, Place, Rvalue, ValueId},
        storage::Location,
    },
    borrow::ownership::{PathId, Paths, paths::Access},
    support::{hash::HashMap, retained::Growth},
    types::{Kind, Store, TypeId},
};
#[derive(Clone, Copy)]
struct Input<'a> {
    types: &'a Store,
    paths: &'a Paths,
    body: &'a Body,
}
#[derive(Clone, Copy)]
struct Construction {
    form: Form,
    selected: u32,
    fields: crate::body::ir::ValueList,
}
pub(super) struct Copy {
    pub source: Range,
    pub destination: Range,
    pub scratch: usize,
}
pub(super) struct Fill {
    pub destination: Range,
    pub value: bool,
}
struct Update {
    destination: Option<Located>,
    expose: Option<u32>,
    storage: Option<bool>,
}
pub(super) struct Row {
    pub destination: Option<Located>,
    pub copies: Range,
    pub fills: Range,
    pub expose: Option<u32>,
    pub storage: Option<bool>,
}
pub(super) struct Transfers {
    pub rows: HashMap<Location, Row>,
    pub copies: Vec<Copy>,
    pub fills: Vec<Fill>,
    pub scratch: usize,
}
impl Transfers {
    pub(super) fn new(
        tags: &mut Tags,
        types: &Store,
        paths: &Paths,
        body: &Body,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<Self, &'static str> {
        let mut this = Self {
            rows: HashMap::default(),
            copies: Vec::new(),
            fills: Vec::new(),
            scratch: 0,
        };
        for (index, block) in body.blocks.iter().enumerate() {
            for before in block.start..block.end {
                tags.parts.charge(1)?;
                let action = body
                    .statements
                    .get(before as usize)
                    .ok_or("cleanup tag statement is missing")?
                    .action;
                let copies = parts::compact(this.copies.len())?;
                let fills = parts::compact(this.fills.len())?;
                let Update {
                    destination,
                    expose,
                    storage,
                } = this.action(tags, Input { types, paths, body }, action, position)?;
                if destination.is_none() && expose.is_none() {
                    continue;
                }
                let location = Location::At {
                    block: BlockId(parts::compact(index)?),
                    before,
                };
                tags.parts.pay(Growth::map::<Location, Row>(
                    this.rows.len(),
                    this.rows.capacity(),
                    1,
                ))?;
                this.rows
                    .try_reserve(1)
                    .map_err(|_| "cleanup tag transfer index allocation failed")?;
                let row = Row {
                    destination,
                    expose,
                    storage,
                    copies: Range {
                        start: copies,
                        end: parts::compact(this.copies.len())?,
                    },
                    fills: Range {
                        start: fills,
                        end: parts::compact(this.fills.len())?,
                    },
                };
                let mut scratch = 0_usize;
                for copy in &mut this.copies[copies as usize..] {
                    copy.scratch = scratch;
                    scratch = scratch
                        .checked_add((copy.source.end - copy.source.start) as usize)
                        .ok_or("cleanup tag scratch size overflow")?;
                }
                this.scratch = this.scratch.max(scratch);
                if this.rows.insert(location, row).is_some() {
                    return Err("cleanup tag statement has multiple owners");
                }
            }
        }
        tags.parts.pay(Growth {
            bytes: this.scratch,
            work: this.scratch,
        })?;
        Ok(this)
    }
    fn action(
        &mut self,
        tags: &mut Tags,
        input: Input<'_>,
        action: Action,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<Update, &'static str> {
        let Input { types, paths, body } = input;
        match action {
            Action::Evaluate(value) => {
                let source = body
                    .values
                    .get(value.0 as usize)
                    .ok_or("cleanup tag value is missing")?;
                let expose = match source.kind {
                    Rvalue::Address(place) => Self::exposure(tags, paths, place, true)?,
                    Rvalue::Borrow { place, .. } => Self::exposure(
                        tags,
                        paths,
                        place,
                        matches!(
                            types.kind(source.ty),
                            Some(Kind::Reference { mutable: true, .. })
                        ),
                    )?,
                    _ => None,
                };
                let destination = paths
                    .temporary_path(value)
                    .and_then(|path| tags.locations.get(&path).copied());
                if let Some(destination) = destination {
                    self.value(tags, input, value, destination, position)?;
                }
                Ok(Update {
                    destination,
                    expose,
                    storage: None,
                })
            }
            Action::Store { place, value } => {
                let destination = tags.place(paths, place)?;
                if let Some(destination) = destination {
                    let source = paths
                        .temporary_path(value)
                        .and_then(|path| tags.locations.get(&path).copied());
                    self.copy(
                        tags,
                        types,
                        source.map(|s| s.coordinate),
                        destination.coordinate,
                    )?;
                }
                Ok(Update {
                    destination,
                    expose: None,
                    storage: None,
                })
            }
            Action::StorageLive(local) | Action::StorageDead(local) => {
                let destination = tags.locations.get(&PathId(local.0)).copied();
                let initialized = matches!(action, Action::StorageLive(_))
                    && body.locals.get(local.0 as usize).is_some_and(|local| {
                        matches!(local.kind, LocalKind::Parameter(_) | LocalKind::Variadic)
                    });
                if let Some(destination) = destination {
                    self.fill(
                        tags,
                        tags.nodes[destination.coordinate.0 as usize].tags,
                        initialized,
                    )?;
                }
                Ok(Update {
                    destination,
                    expose: None,
                    storage: Some(initialized),
                })
            }
            _ => Ok(Update {
                destination: None,
                expose: None,
                storage: None,
            }),
        }
    }
    fn exposure(
        tags: &Tags,
        paths: &Paths,
        place: Place,
        mutable: bool,
    ) -> Result<Option<u32>, &'static str> {
        let info = paths.place(place)?;
        let Some(path) = info
            .path
            .filter(|_| info.access == Access::Owned && !info.empty)
        else {
            return Ok(None);
        };
        let Some(root) = tags.origins.get(&path).copied() else {
            return Ok(None);
        };
        let part = tags.nodes[tags.roots[root as usize].coordinate.0 as usize].part;
        Ok((mutable || tags.parts.entries[part.0 as usize].interior).then_some(root))
    }
    fn value(
        &mut self,
        tags: &mut Tags,
        input: Input<'_>,
        value: ValueId,
        destination: Located,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<(), &'static str> {
        let Input { types, paths, body } = input;
        let value = body
            .values
            .get(value.0 as usize)
            .ok_or("cleanup tag constructor is missing")?;
        match value.kind {
            Rvalue::Read(place) => self.copy(
                tags,
                types,
                tags.place(paths, place)?.map(|p| p.coordinate),
                destination.coordinate,
            ),
            Rvalue::Aggregate { variant, fields } => {
                let part = &tags.parts.entries
                    [tags.nodes[destination.coordinate.0 as usize].part.0 as usize];
                let form = part.form;
                let selected = match (form, variant) {
                    (Form::Nominal { .. }, Some(variant)) => {
                        tags.parts
                            .charge((part.variants.end - part.variants.start) as usize + 1)?;
                        let MemberPosition::Variant(ordinal) =
                            position(value.ty, MemberQuery::Constructor(variant))?
                        else {
                            return Err("cleanup tag constructor lost actual variant identity");
                        };
                        ordinal
                    }
                    (Form::Tuple | Form::Array, None) => 0,
                    (Form::Leaf, _) => {
                        return self.fill(
                            tags,
                            tags.nodes[destination.coordinate.0 as usize].tags,
                            true,
                        );
                    }
                    _ => return Err("cleanup tag constructor has another structural owner"),
                };
                self.constructor(
                    tags,
                    input,
                    destination.coordinate,
                    Construction {
                        form,
                        selected,
                        fields,
                    },
                )
            }
            Rvalue::Repeat { value, .. } => {
                let Some(child) = tags.child(destination.coordinate, Step::Element)? else {
                    return Ok(());
                };
                let source = paths
                    .temporary_path(value)
                    .and_then(|path| tags.locations.get(&path))
                    .map(|p| p.coordinate);
                self.copy(tags, types, source, child)
            }
            _ => self.fill(
                tags,
                tags.nodes[destination.coordinate.0 as usize].tags,
                true,
            ),
        }
    }
    fn constructor(
        &mut self,
        tags: &mut Tags,
        input: Input<'_>,
        destination: Coordinate,
        construction: Construction,
    ) -> Result<(), &'static str> {
        let Input { types, paths, body } = input;
        let Construction {
            form,
            selected,
            fields,
        } = construction;
        let node = &tags.nodes[destination.0 as usize];
        let part = &tags.parts.entries[node.part.0 as usize];
        let variant = part
            .variants
            .start
            .checked_add(selected)
            .filter(|&v| v < part.variants.end)
            .ok_or("cleanup tag constructor variant exceeds actual header")?;
        let descriptor = tags
            .parts
            .variants
            .get(variant as usize)
            .ok_or("cleanup tag constructor descriptor is missing")?
            .fields;
        let values = body
            .arguments
            .get(fields.start as usize..fields.end as usize)
            .ok_or("cleanup tag constructor arguments are missing")?;
        if !matches!(form, Form::Array)
            && values.len() != (descriptor.end - descriptor.start) as usize
        {
            return Err("cleanup tag constructor changed its actual field count");
        }
        let own = node.own_tags;
        let need = node.own_need;
        self.fill(tags, need, true)?;
        if matches!(form, Form::Nominal { enumeration: true }) {
            let bit = own
                .start
                .checked_add(selected)
                .filter(|&bit| bit < own.end)
                .ok_or("cleanup tag constructor exceeds actual tag partition")?;
            self.fill(
                tags,
                Range {
                    start: bit,
                    end: bit + 1,
                },
                true,
            )?;
        }
        tags.parts.charge(values.len())?;
        for (ordinal, &value) in values.iter().enumerate() {
            body.values
                .get(value.0 as usize)
                .ok_or("cleanup tag argument has no typed value")?;
            let step = match form {
                Form::Array => Step::Element,
                Form::Tuple => Step::Tuple(parts::compact(ordinal)?),
                Form::Nominal { .. } => Step::Nominal {
                    variant: selected,
                    field: parts::compact(ordinal)?,
                },
                Form::Leaf => return Err("cleanup tag constructor is a leaf"),
            };
            let Some(child) = tags.child(destination, step)? else {
                continue;
            };
            let source = paths
                .temporary_path(value)
                .and_then(|path| tags.locations.get(&path))
                .map(|source| source.coordinate);
            self.copy(tags, types, source, child)?;
        }
        Ok(())
    }
    fn fill(
        &mut self,
        tags: &mut Tags,
        destination: Range,
        value: bool,
    ) -> Result<(), &'static str> {
        if destination.start == destination.end {
            return Ok(());
        }
        tags.parts.pay(Growth::vector::<Fill>(
            self.fills.len(),
            self.fills.capacity(),
            1,
        ))?;
        self.fills
            .try_reserve(1)
            .map_err(|_| "cleanup tag fill allocation failed")?;
        self.fills.push(Fill { destination, value });
        Ok(())
    }
    fn copy(
        &mut self,
        tags: &mut Tags,
        types: &Store,
        source: Option<Coordinate>,
        destination: Coordinate,
    ) -> Result<(), &'static str> {
        let target = &tags.nodes[destination.0 as usize];
        let range = target.tags;
        if range.start == range.end {
            return Ok(());
        }
        let Some(source) = source else {
            return self.fill(tags, range, true);
        };
        let source = &tags.nodes[source.0 as usize];
        if source.part != target.part {
            tags.parts.charge(1)?;
            let a = tags.parts.entries[source.part.0 as usize].ty;
            let b = tags.parts.entries[target.part.0 as usize].ty;
            if !matches!((types.kind(a),types.kind(b)),(Some(Kind::Adt(x,ax)),Some(Kind::Adt(y,ay)))
                if x==y && types.region_args(ax).is_some() && types.region_args(ay).is_some())
            {
                return self.fill(tags, range, true);
            }
        }
        let source = source.tags;
        if source.end - source.start != range.end - range.start {
            return Err("cleanup tag copy changed its structural size");
        }
        tags.parts.pay(Growth::vector::<Copy>(
            self.copies.len(),
            self.copies.capacity(),
            1,
        ))?;
        self.copies
            .try_reserve(1)
            .map_err(|_| "cleanup tag copy allocation failed")?;
        self.copies.push(Copy {
            source,
            destination: range,
            scratch: 0,
        });
        Ok(())
    }
}
