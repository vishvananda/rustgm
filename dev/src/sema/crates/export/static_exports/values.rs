//! Encode each completed typed value once, retaining named address identities.
use super::{Declarations, Error, OriginValue, Prepared, error};
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    sema::{
        crates::type_transport,
        values::{Constant, Origin, Reader, View},
    },
    support::crates::{
        RecordRef, Static, StaticRef, Storage, StorageOrigin, StoredKind, StoredProjection,
        StoredValue, ValueRef,
    },
    types::{ItemId, Kind, TypeId},
};

impl Prepared {
    pub(in crate::sema::crates::export) fn encode(
        self,
        facts: &Declarations,
        types: &mut type_transport::Export,
        records: &HashMap<ItemId, RecordRef>,
    ) -> Result<Option<Storage>, Error> {
        if self.entries.is_empty() {
            return Ok(None);
        }
        let definitions = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                u32::try_from(i)
                    .map(|i| (entry.definition, StaticRef(i)))
                    .map_err(|_| error("too many exported static identities"))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let mut builder = Values {
            facts,
            types,
            records,
            definitions,
            nodes: Vec::new(),
            remap: HashMap::default(),
            steps: 0,
        };
        let mut entries = Vec::with_capacity(self.entries.len());
        for entry in self.entries {
            let ty = builder.types.root(facts, entry.ty, records)?;
            let origin = match entry.origin {
                OriginValue::Defined(value) => {
                    StorageOrigin::Defined(builder.root(entry.ty, value)?)
                }
                OriginValue::Foreign { unsafe_ } => StorageOrigin::Foreign { unsafe_ },
            };
            entries.push(
                Static::new(
                    entry.external,
                    ty,
                    entry.symbol,
                    entry.public,
                    entry.mutable,
                    origin,
                )
                .map_err(error)?,
            );
        }
        Storage::new(entries, builder.nodes)
            .map(Some)
            .map_err(error)
    }
}

struct Values<'a> {
    facts: &'a Declarations,
    types: &'a mut type_transport::Export,
    records: &'a HashMap<ItemId, RecordRef>,
    definitions: HashMap<DefId, StaticRef>,
    nodes: Vec<StoredValue>,
    remap: HashMap<(TypeId, Constant), ValueRef>,
    steps: usize,
}
impl Values<'_> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err(error("metadata stored values exceed profile budget"));
        }
        Ok(())
    }
    fn root(&mut self, ty: TypeId, value: Constant) -> Result<ValueRef, Error> {
        let mut work = vec![(ty, value, false)];
        while let Some((ty, value, finish)) = work.pop() {
            self.charge(1)?;
            if self.remap.contains_key(&(ty, value)) {
                continue;
            }
            let reader = Reader::new(self.facts);
            let kind = match reader.view(value).map_err(error)? {
                View::Scalar(_) => StoredKind::Scalar(super::super::named_values::value_bits(
                    super::super::scalar(self.facts, ty)?,
                    reader.view(value).map_err(error)?,
                )?),
                View::Repeat {
                    value: child,
                    length,
                } => {
                    let Some(Kind::Array {
                        element,
                        length: expected,
                    }) = self.facts.types.kind(ty)
                    else {
                        return Err(error("metadata repeat requires an array type"));
                    };
                    if length != expected {
                        return Err(error("metadata repeat differs from its array length"));
                    }
                    if !finish {
                        work.push((ty, value, true));
                        work.push((element, child, false));
                        continue;
                    }
                    StoredKind::Repeat(
                        *self
                            .remap
                            .get(&(element, child))
                            .ok_or_else(|| error("missing exported repeated element"))?,
                    )
                }
                View::Bytes(_) | View::ByteRange { .. } => {
                    let bytes = reader.stored_bytes(ty, value).map_err(error)?;
                    self.charge(bytes.len())?;
                    StoredKind::Bytes(bytes.into())
                }
                View::Product { length, variant } => {
                    let length = usize::try_from(length)
                        .map_err(|_| error("metadata product length exceeds host index"))?;
                    self.charge(length)?;
                    if !finish {
                        work.push((ty, value, true));
                        for index in (0..length as u64).rev() {
                            work.push((
                                reader.stored_child(ty, variant, index).map_err(error)?,
                                reader.field(value, index).map_err(error)?,
                                false,
                            ));
                        }
                        continue;
                    }
                    let fields = (0..length as u64)
                        .map(|index| {
                            let child = (
                                reader.stored_child(ty, variant, index).map_err(error)?,
                                reader.field(value, index).map_err(error)?,
                            );
                            self.remap
                                .get(&child)
                                .copied()
                                .ok_or_else(|| error("missing exported product field"))
                        })
                        .collect::<Result<Box<[_]>, _>>()?;
                    StoredKind::Product(fields)
                }
                View::Reference(location) => self.reference(value, location)?,
                _ => {
                    return Err(error(
                        "static initializer value is outside the metadata profile",
                    ));
                }
            };
            let id = ValueRef(
                u32::try_from(self.nodes.len()).map_err(|_| error("too many metadata values"))?,
            );
            self.nodes.push(StoredValue {
                ty: self.types.root(self.facts, ty, self.records)?,
                kind,
            });
            self.remap.insert((ty, value), id);
        }
        self.remap
            .get(&(ty, value))
            .copied()
            .ok_or_else(|| error("missing exported static value"))
    }
    fn reference(
        &mut self,
        value: Constant,
        mut location: crate::sema::values::LocationId,
    ) -> Result<StoredKind, Error> {
        if value.metadata().is_some() {
            return Err(error(
                "wide static reference metadata is outside the profile",
            ));
        }
        let mut path = Vec::new();
        let definition = loop {
            self.charge(1)?;
            match Reader::new(self.facts)
                .location(location)
                .map_err(error)?
                .origin
            {
                Origin::Static(definition) => break definition,
                Origin::Projection { base, index } => {
                    let reader = Reader::new(self.facts);
                    let parent = reader.location(base).map_err(error)?.ty;
                    let child = reader.stored_child(parent, None, index).map_err(error)?;
                    if child != reader.location(location).map_err(error)?.ty {
                        return Err(error(
                            "metadata static projection has a different stored type",
                        ));
                    }
                    path.push(match self.facts.types.kind(parent) {
                        Some(Kind::Array { .. }) => StoredProjection::Element(index),
                        Some(Kind::Adt(_, _)) => StoredProjection::Field(
                            u32::try_from(index)
                                .map_err(|_| error("metadata field ordinal exceeds 32 bits"))?,
                        ),
                        _ => {
                            return Err(error(
                                "metadata static projection lacks aggregate storage",
                            ));
                        }
                    });
                    location = base;
                }
                _ => {
                    return Err(error(
                        "unnamed or reinterpreted static backing is outside the profile",
                    ));
                }
            }
        };
        path.reverse();
        let target = *self
            .definitions
            .get(&definition)
            .ok_or_else(|| error("missing exported static reference target"))?;
        if path
            .iter()
            .any(|p| matches!(p, StoredProjection::Element(_)))
        {
            Ok(StoredKind::ProjectedReference {
                target,
                path: path.into_boxed_slice(),
            })
        } else {
            let fields = path
                .into_iter()
                .map(|p| match p {
                    StoredProjection::Field(index) => Ok(index),
                    StoredProjection::Element(_) => {
                        Err(error("unexpected metadata element projection"))
                    }
                })
                .collect::<Result<Box<[_]>, _>>()?;
            Ok(StoredKind::Reference { target, fields })
        }
    }
}
