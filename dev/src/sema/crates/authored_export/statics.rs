//! Source static metadata preserves a single provider-owned native allocation.
use super::{
    Context, CrateIdentity, DeclarationRef, Declarations, DefId, Namespace, Report, Stage, g, s,
};
use crate::{
    sema::{
        foreign::Global,
        members::{Field, VariantId},
        model::InitializerKind,
        values::{Constant, Failure, LocationId, Reader, View},
    },
    support::linkage::{Linkage, Naming},
    syntax::item::Safety,
    types::{GenericArgs, Kind, TypeId},
};

pub(super) struct StaticDraft {
    headers: Vec<HeaderDraft>,
    values: Vec<u8>,
}

struct HeaderDraft {
    declaration: DeclarationRef,
    ty: g::TypeRef,
    initializer: InitializerDraft,
}

enum InitializerDraft {
    Scalars(g::Range),
    Promotion(LocationId),
    ProviderOwned,
}

pub(super) fn require_definition(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Result<(), &'static str> {
    if facts.thread_local_static(definition) {
        return Err("source static transport requires process-wide storage");
    }
    let source = &report.resolution.definitions[definition.0 as usize];
    let initializer = facts
        .initializer_index(definition)
        .and_then(|index| facts.initializers.get(index))
        .ok_or("source static requires its own checked initializer")?;
    if source.namespace != Namespace::Value
        || source.node != Some(initializer.node)
        || !initializer.type_checked()
        || initializer.kind != (InitializerKind::Static { mutable: false })
        || facts.definitions[definition.0 as usize] != Some(initializer.ty)
        || facts
            .foreign
            .as_deref()
            .and_then(|registry| registry.global(definition))
            .is_some()
    {
        return Err("source static requires its own immutable Rust allocation");
    }
    Ok(())
}

impl Stage {
    /// The static's completed initializer already travels in its data column.
    pub(crate) fn evaluated_static(&mut self, definition: DefId) -> Result<bool, &'static str> {
        if !self.declaration_policies() {
            return Ok(false);
        }
        self.charge(1)?;
        Ok(self.definitions[definition.0 as usize].is_some_and(|row| {
            self.rows.declarations[row.0 as usize].definition.kind == s::DefinitionKind::Static
        }))
    }

    pub(crate) fn static_declaration(
        &mut self,
        definition: DefId,
    ) -> Result<DeclarationRef, &'static str> {
        let row = self.declaration(definition)?;
        if self.rows.declarations[row.0 as usize].definition.kind != s::DefinitionKind::Static {
            return Err("source static place lacks its actual allocation declaration");
        }
        Ok(row)
    }

    pub(super) fn static_headers(
        &mut self,
        facts: &mut Declarations,
        identity: &CrateIdentity,
        declarations: Vec<(DeclarationRef, DefId)>,
    ) -> Result<Option<StaticDraft>, &'static str> {
        if declarations.is_empty() {
            return Ok(None);
        }
        self.reserve(
            size_of::<s::StaticColumns>().saturating_add(
                declarations
                    .len()
                    .saturating_mul(size_of::<s::StaticHeader>() + size_of::<HeaderDraft>()),
            ),
        )?;
        let mut draft = StaticDraft {
            headers: Vec::with_capacity(declarations.len()),
            values: Vec::new(),
        };
        for (row, definition) in declarations {
            let ty = facts.definitions[definition.0 as usize]
                .ok_or("source static lost its actual type")?;
            let raw = self.ty(
                facts,
                ty,
                Context {
                    owner: row,
                    frame: None,
                    body: None,
                },
            )?;
            let value = Reader::new(facts)
                .definition(definition)
                .ok_or("source static requires its completed value")?;
            let initializer = if matches!(
                facts.types.kind(ty),
                Some(Kind::Adt(..) | Kind::Pointer { .. })
            ) && let Some((location, work)) =
                super::super::relocated_initializers::checked_pointer(facts, ty, value)?
            {
                self.charge(work)?;
                InitializerDraft::Promotion(location)
            } else {
                let start = offset(&draft.values)?;
                if self.try_scalar_aggregate_bytes(facts, ty, value, &mut draft.values)? {
                    InitializerDraft::Scalars(g::Range {
                        start,
                        end: offset(&draft.values)?,
                    })
                } else {
                    draft.values.truncate(start as usize);
                    InitializerDraft::ProviderOwned
                }
            };
            draft.headers.push(HeaderDraft {
                declaration: row,
                ty: raw,
                initializer,
            });
            self.install_static_symbol(facts, identity, row, definition, ty)?;
        }
        Ok(Some(draft))
    }

    /// Evaluated constant data may finish after callable headers; resolve its
    /// actual allocation here while keeping the static type recipes in order.
    pub(super) fn complete_static_headers(
        &mut self,
        facts: &Declarations,
        draft: Option<StaticDraft>,
    ) -> Result<(), &'static str> {
        let Some(draft) = draft else {
            return Ok(());
        };
        self.charge(draft.headers.len())?;
        let headers = draft
            .headers
            .into_iter()
            .map(|row| {
                let initializer = match row.initializer {
                    InitializerDraft::Scalars(range) => s::StaticInitializer::Scalars(range),
                    InitializerDraft::Promotion(location) => {
                        if let Some(binding) = facts
                            .crates
                            .as_deref()
                            .and_then(|linkage| linkage.promoted_allocation(location))
                            .filter(|binding| !binding.imported)
                        {
                            s::StaticInitializer::Promotion(binding.anchor)
                        } else {
                            s::StaticInitializer::ProviderOwned
                        }
                    }
                    InitializerDraft::ProviderOwned => s::StaticInitializer::ProviderOwned,
                };
                Ok(s::StaticHeader {
                    declaration: row.declaration,
                    ty: row.ty,
                    initializer,
                })
            })
            .collect::<Result<_, &'static str>>()?;
        self.cold()?.statics = Some(Box::new(s::StaticColumns {
            headers,
            values: draft.values,
        }));
        Ok(())
    }

    fn install_static_symbol(
        &mut self,
        facts: &mut Declarations,
        identity: &CrateIdentity,
        row: DeclarationRef,
        definition: DefId,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        let symbol = super::super::static_symbol(identity, row);
        self.reserve(
            symbol
                .capacity()
                .saturating_add(crate::support::retained::new_map::<DefId, String>(1))
                .saturating_add(crate::support::retained::new_map::<DefId, Global>(1)),
        )?;
        let linkage = facts.crates.get_or_insert_default();
        if linkage
            .symbols
            .get(&definition)
            .is_some_and(|prior| prior != &symbol)
        {
            return Err("source static changed its canonical native symbol");
        }
        linkage.symbols.insert(definition, symbol);
        facts.foreign.get_or_insert_default().insert_global(
            definition,
            Global {
                ty,
                mutable: false,
                safety: Safety::Safe,
                naming: Naming::Rust,
                linkage: Linkage::Export,
                weak: None,
            },
        )
    }

    pub(super) fn scalar_aggregate_bytes(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        value: Constant,
        bytes: &mut Vec<u8>,
    ) -> Result<(), &'static str> {
        if !self.try_scalar_aggregate_bytes(facts, ty, value, bytes)? {
            return Err("source static initializer requires its closed scalar aggregate type");
        }
        Ok(())
    }

    fn try_scalar_aggregate_bytes(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        value: Constant,
        bytes: &mut Vec<u8>,
    ) -> Result<bool, &'static str> {
        let reader = Reader::new(facts);
        let mut pending = vec![Visit::Value(ty, value)];
        while let Some(visit) = pending.pop() {
            self.charge(1)?;
            self.scratch(
                pending
                    .len()
                    .saturating_add(3)
                    .saturating_mul(4 * size_of::<Visit<'_>>()),
            )?;
            match visit {
                Visit::Value(ty, value) => {
                    let (children, variant) = match facts.types.kind(ty) {
                        Some(Kind::Primitive(_)) => {
                            let (bits, width) = reader
                                .encode_primitive(ty, value)
                                .map_err(Failure::message)?;
                            let raw = bits.to_le_bytes();
                            let field = raw
                                .get(..width)
                                .ok_or("source static primitive width exceeds its value")?;
                            self.reserve(4 * field.len())?;
                            bytes.extend_from_slice(field);
                            continue;
                        }
                        Some(Kind::Tuple(list)) => (
                            Children::Tuple(
                                facts
                                    .types
                                    .list(list)
                                    .ok_or("source static lost its tuple type")?,
                            ),
                            None,
                        ),
                        Some(Kind::Array { element, length }) => {
                            if matches!(
                                reader.view(value)?,
                                View::Bytes(_) | View::ByteRange { .. }
                            ) {
                                let slice = reader.stored_bytes(ty, value)?;
                                self.reserve(4 * slice.len())?;
                                bytes.extend_from_slice(slice);
                                continue;
                            }
                            (Children::Array(element, length), None)
                        }
                        Some(Kind::Adt(owner, GenericArgs::EMPTY))
                            if self
                                .definitions
                                .get(owner.0 as usize)
                                .copied()
                                .flatten()
                                .is_some()
                                && facts.aggregates.get(owner).is_some_and(|record| {
                                    record.kind == crate::types::aggregate::Kind::Struct
                                }) =>
                        {
                            let [variant] = facts.members.variants(owner) else {
                                return Ok(false);
                            };
                            let Some(fields) = facts.members.fields(*variant) else {
                                return Ok(false);
                            };
                            (Children::Struct(fields), Some(*variant))
                        }
                        _ => return Ok(false),
                    };
                    require_shape(reader, value, children.len(), variant)?;
                    pending.push(Visit::Children(children, value, 0));
                }
                Visit::Children(children, value, index) => {
                    if let Some(ty) = children.get(index) {
                        pending.push(Visit::Children(children, value, index + 1));
                        pending.push(Visit::Value(ty, reader.field(value, index)?));
                    }
                }
            }
        }
        Ok(true)
    }
}

#[derive(Clone, Copy)]
enum Children<'a> {
    Tuple(&'a [TypeId]),
    Struct(&'a [Field]),
    Array(TypeId, u64),
}

impl Children<'_> {
    fn len(self) -> u64 {
        match self {
            Self::Tuple(fields) => fields.len() as u64,
            Self::Struct(fields) => fields.len() as u64,
            Self::Array(_, count) => count,
        }
    }

    fn get(self, index: u64) -> Option<TypeId> {
        match self {
            Self::Tuple(fields) => usize::try_from(index)
                .ok()
                .and_then(|index| fields.get(index))
                .copied(),
            Self::Struct(fields) => usize::try_from(index)
                .ok()
                .and_then(|index| fields.get(index))
                .map(|field| field.ty),
            Self::Array(element, count) => (index < count).then_some(element),
        }
    }
}

enum Visit<'a> {
    Value(TypeId, Constant),
    Children(Children<'a>, Constant, u64),
}

fn require_shape(
    reader: Reader<'_>,
    value: Constant,
    expected: u64,
    expected_variant: Option<VariantId>,
) -> Result<(), &'static str> {
    match reader.view(value)? {
        View::Product { length, variant } if length == expected && variant == expected_variant => {
            Ok(())
        }
        View::Repeat { length, .. } if expected_variant.is_none() && length == expected => Ok(()),
        _ => Err("source static value differs from its canonical aggregate shape"),
    }
}

fn offset(bytes: &[u8]) -> Result<u32, &'static str> {
    u32::try_from(bytes.len()).map_err(|_| "source static initializer byte table overflow")
}
