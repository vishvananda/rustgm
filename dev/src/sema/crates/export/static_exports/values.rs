//! Encode each completed typed value once, retaining named address identities.
use super::{Declarations, Error, OriginValue, Prepared, error};
use crate::{
    resolve::model::DefId,
    sema::{
        crates::type_transport,
        values::{Constant, Origin, Reader, View},
    },
    support::crates::{
        RecordRef, Static, StaticRef, Storage, StorageOrigin, StoredKind, StoredValue, ValueRef,
    },
    types::{ItemId, TypeId},
};
use std::collections::HashMap;

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
            remap: HashMap::new(),
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
        let mut fields = Vec::new();
        let definition = loop {
            self.charge(1)?;
            match Reader::new(self.facts)
                .location(location)
                .map_err(error)?
                .origin
            {
                Origin::Static(definition) => break definition,
                Origin::Projection { base, index } => {
                    fields.push(
                        u32::try_from(index)
                            .map_err(|_| error("metadata field ordinal exceeds 32 bits"))?,
                    );
                    location = base;
                }
                _ => {
                    return Err(error(
                        "unnamed or reinterpreted static backing is outside the profile",
                    ));
                }
            }
        };
        fields.reverse();
        Ok(StoredKind::Reference {
            target: *self
                .definitions
                .get(&definition)
                .ok_or_else(|| error("missing exported static reference target"))?,
            fields: fields.into_boxed_slice(),
        })
    }
}
