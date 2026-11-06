//! One checked statement index supplies both tag dataflow and replay transfers.
use super::qterm::Input;
use super::{
    parts::{self, Form, Range},
    qterm::{Aliases, Compiler, Source, Terms},
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
struct Construction {
    form: Form,
    selected: u32,
    fields: crate::body::ir::ValueList,
    ty: TypeId,
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
    expose: Option<(u32, TypeId)>,
    storage: Option<bool>,
    source: Source,
}
pub(super) struct Row {
    pub destination: Option<Located>,
    pub copies: Range,
    pub fills: Range,
    pub expose: Option<(u32, TypeId)>,
    pub storage: Option<bool>,
    source: Source,
    pub qualification: Range,
}
pub(super) struct Candidate {
    pub value: ValueId,
    pub variant: u32,
    pub callee: TypeId,
    pub signature: TypeId,
    pub result: TypeId,
}
pub(super) struct Transfers {
    pub candidates: Vec<Candidate>,
    pub rows: HashMap<Location, Row>,
    pub copies: Vec<Copy>,
    pub fills: Vec<Fill>,
    pub scratch: usize,
    pub terms: Terms,
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
            candidates: Vec::new(),
            rows: HashMap::default(),
            copies: Vec::new(),
            fills: Vec::new(),
            scratch: 0,
            terms: Terms::default(),
        };
        let mut aliases = Aliases::default();
        for (index, block) in body.blocks.iter().enumerate() {
            for before in block.start..block.end {
                tags.parts.charge(1)?;
                let action = body
                    .statements
                    .get(before as usize)
                    .ok_or("cleanup tag statement is missing")?
                    .action;
                aliases.observe(&tags.parts, Input { types, paths, body }, action)?;
                let copies = parts::compact(this.copies.len())?;
                let fills = parts::compact(this.fills.len())?;
                let Update {
                    destination,
                    expose,
                    storage,
                    source,
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
                    source,
                    qualification: Range::default(),
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
        let mut compiler = Compiler::default();
        for row in this.rows.values_mut() {
            row.qualification = this.terms.compile(
                tags,
                Input { types, paths, body },
                &aliases,
                &mut compiler,
                row.source,
            )?;
        }
        this.scratch = this
            .scratch
            .checked_add(1)
            .ok_or("qualification scratch size overflow")?;
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
                    Rvalue::Reborrow { place, mutable } => {
                        Self::exposure(tags, paths, place, mutable)?
                    }
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
                let qualification = if let Some(destination) = destination {
                    self.value(tags, input, value, destination, position)?
                } else {
                    Source::Clear
                };
                let expose = exposure_type(body, source.kind, expose)?;
                Ok(Update {
                    destination,
                    expose,
                    storage: None,
                    source: qualification,
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
                    source: Source::Value(value),
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
                    source: if initialized {
                        Source::Type(
                            body.locals
                                .get(local.0 as usize)
                                .ok_or("qualification parameter is missing")?
                                .ty,
                        )
                    } else {
                        Source::Clear
                    },
                })
            }
            _ => Ok(Update {
                destination: None,
                expose: None,
                storage: None,
                source: Source::Clear,
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
    ) -> Result<Source, &'static str> {
        let Input { types, paths, body } = input;
        let identity = value;
        let value = body
            .values
            .get(value.0 as usize)
            .ok_or("cleanup tag constructor is missing")?;
        match value.kind {
            Rvalue::Read(place) | Rvalue::Reborrow { place, .. } => {
                self.copy(
                    tags,
                    types,
                    tags.place(paths, place)?.map(|p| p.coordinate),
                    destination.coordinate,
                )?;
                Ok(Source::Place(place))
            }
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
                        self.fill(
                            tags,
                            tags.nodes[destination.coordinate.0 as usize].tags,
                            true,
                        )?;
                        return Ok(Source::Type(value.ty));
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
                        ty: value.ty,
                    },
                )
            }
            Rvalue::Call { .. } => self.call(tags, input, identity, destination, position),
            Rvalue::Constrain(value) => {
                let source = paths
                    .temporary_path(value)
                    .and_then(|path| tags.locations.get(&path))
                    .map(|p| p.coordinate);
                self.copy(tags, types, source, destination.coordinate)?;
                Ok(Source::Value(value))
            }
            Rvalue::Repeat { value, .. } => {
                let Some(child) = tags.child(destination.coordinate, Step::Element)? else {
                    return Ok(Source::Value(value));
                };
                let source = paths
                    .temporary_path(value)
                    .and_then(|path| tags.locations.get(&path))
                    .map(|p| p.coordinate);
                self.copy(tags, types, source, child)?;
                Ok(Source::Value(value))
            }
            _ => {
                self.fill(
                    tags,
                    tags.nodes[destination.coordinate.0 as usize].tags,
                    true,
                )?;
                Ok(Source::Type(value.ty))
            }
        }
    }
    fn call(
        &mut self,
        tags: &mut Tags,
        input: Input<'_>,
        identity: ValueId,
        destination: Located,
        position: &impl Fn(TypeId, MemberQuery) -> Result<MemberPosition, &'static str>,
    ) -> Result<Source, &'static str> {
        let Input { types, body, .. } = input;
        let value = body
            .values
            .get(identity.0 as usize)
            .ok_or("cleanup constructor value is missing")?;
        let Rvalue::Call {
            callee,
            signature,
            arguments,
        } = value.kind
        else {
            return Err("cleanup constructor call changed its transfer");
        };
        let callee = body
            .values
            .get(callee.0 as usize)
            .ok_or("cleanup constructor callee is missing")?;
        if !matches!((types.kind(callee.ty), callee.kind),
                    (Some(Kind::FunctionItem {item, ..}), Rvalue::Function(definition)) if item.0 == definition.0)
        {
            self.fill(
                tags,
                tags.nodes[destination.coordinate.0 as usize].tags,
                true,
            )?;
            return Ok(Source::Type(value.ty));
        }
        let callee = callee.ty;
        let selected = position(value.ty, MemberQuery::ConstructorCall(callee))?;
        let MemberPosition::Variant(selected) = selected else {
            self.fill(
                tags,
                tags.nodes[destination.coordinate.0 as usize].tags,
                true,
            )?;
            return Ok(Source::Type(value.ty));
        };
        tags.parts.pay(Growth::vector::<Candidate>(
            self.candidates.len(),
            self.candidates.capacity(),
            1,
        ))?;
        self.candidates
            .try_reserve(1)
            .map_err(|_| "cleanup constructor candidate allocation failed")?;
        self.candidates.push(Candidate {
            value: identity,
            variant: selected,
            callee,
            signature,
            result: value.ty,
        });
        let form =
            tags.parts.entries[tags.nodes[destination.coordinate.0 as usize].part.0 as usize].form;
        self.constructor(
            tags,
            input,
            destination.coordinate,
            Construction {
                form,
                selected,
                fields: arguments,
                ty: value.ty,
            },
        )
    }
    fn constructor(
        &mut self,
        tags: &mut Tags,
        input: Input<'_>,
        destination: Coordinate,
        construction: Construction,
    ) -> Result<Source, &'static str> {
        let Input { types, paths, body } = input;
        let Construction {
            form,
            selected,
            fields,
            ty,
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
        let nonstructural = part.need == super::parts::Need::OwnDrop || matches!(form, Form::Leaf);
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
        Ok(if nonstructural {
            Source::Type(ty)
        } else {
            Source::Fields(fields)
        })
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

fn exposure_type(
    body: &Body,
    kind: Rvalue,
    root: Option<u32>,
) -> Result<Option<(u32, TypeId)>, &'static str> {
    root.map(|root| {
        let place = match kind {
            Rvalue::Address(place)
            | Rvalue::Borrow { place, .. }
            | Rvalue::Reborrow { place, .. } => Some(place),
            _ => None,
        };
        place
            .and_then(|p| body.place_type(p))
            .map(|ty| (root, ty))
            .ok_or("qualification exposure lost its actual borrowed type")
    })
    .transpose()
}
