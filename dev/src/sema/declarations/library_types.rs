//! Library nominal schemas publish ordinary fields, parameters and constructors directly.
use super::{Checker, DefId, Error, ItemId, State, Type};
use crate::{
    resolve::{
        model::{Access, LibraryNominal},
        unit::NodeKey,
    },
    sema::{
        generics::Parameter,
        members::{Field, Name},
    },
    support::library_types::{LibraryField, LibraryParameter, LibraryTag, LibraryType},
    syntax::item::Shape,
    types::{
        GenericArgs, RegionId, TypeId,
        aggregate::{self, Repr, Representation},
        arguments::Argument,
        parameters::{Parameter as Formal, ParameterKind},
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/collection_declarations.rs"]
mod collection_tests;
impl Checker<'_> {
    pub(super) fn prepare_library_types(&mut self) -> Result<(), Error> {
        for kind in LibraryType::ALL {
            if kind == LibraryType::SliceIter
                && !self.report.resolution.builtin_traits.named_slice_iterator()
            {
                continue;
            }
            if !self
                .report
                .resolution
                .builtin_traits
                .comparison_result_latent(kind)
                && let Some(declaration) = self.report.resolution.library_types.get(kind).cloned()
            {
                self.library_nominal(&declaration)?;
            }
        }
        Ok(())
    }
    pub(super) fn library_nominal(&mut self, declaration: &LibraryNominal) -> Result<(), Error> {
        let prepared = super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .library_nominal(declaration.kind)?;
        for definition in std::iter::once(declaration.definition)
            .chain(declaration.parameters.iter().copied())
            .chain(prepared.constructors.iter().copied())
        {
            self.states[definition.0 as usize] = State::Complete;
        }
        self.register_library_nominal(
            ItemId(declaration.definition.0),
            declaration.origin,
            prepared.kind,
            prepared.repr,
            &prepared.constructors,
        )
    }
}
pub(super) struct PreparedNominal {
    kind: aggregate::Kind,
    repr: Repr,
    constructors: Vec<DefId>,
}
impl super::comparisons::LibraryContracts<'_> {
    fn library_parameters(&mut self, declaration: &LibraryNominal) -> Result<Vec<Argument>, Error> {
        let mut parameters = Vec::new();
        let mut arguments = Vec::new();
        let descriptors = declaration.kind.parameters();
        if declaration.parameters.len() != descriptors.len() {
            return Err(self.error(declaration.origin, "invalid library parameter schema"));
        }
        for (index, (&binding, descriptor)) in
            declaration.parameters.iter().zip(descriptors).enumerate()
        {
            let argument = self.library_parameter(declaration, binding, index, *descriptor)?;
            arguments.push(argument);
            parameters.push(Parameter {
                binding,
                anonymous: false,
                argument,
                default: None,
            });
        }
        self.facts
            .generics
            .parameters
            .insert(declaration.definition, parameters);
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, declaration.definition)
            .map_err(|e| self.error(declaration.origin, e))?;
        Ok(arguments)
    }
    fn library_parameter(
        &mut self,
        declaration: &LibraryNominal,
        binding: DefId,
        index: usize,
        descriptor: LibraryParameter,
    ) -> Result<Argument, Error> {
        if let LibraryParameter::Type { sized, .. } = descriptor {
            let parameter = self
                .facts
                .types
                .intern_parameter(Formal {
                    owner: ItemId(declaration.definition.0),
                    index: u32::try_from(index).map_err(|_| {
                        self.error(declaration.origin, "too many library parameters")
                    })?,
                    kind: ParameterKind::Type { sized },
                })
                .map_err(|e| self.error(declaration.origin, e))?;
            let ty = self.intern(declaration.origin, Type::Parameter(parameter))?;
            self.facts.definitions[binding.0 as usize] = Some(ty);
            return Ok(Argument::Type(ty));
        }
        let count = self
            .facts
            .lifetimes
            .parameters
            .get(&declaration.definition)
            .map_or(0, Vec::len);
        let slot = u16::try_from(count)
            .map_err(|_| self.error(declaration.origin, "too many library lifetimes"))?;
        let region = RegionId::bound(0, slot).map_err(|e| self.error(declaration.origin, e))?;
        self.facts
            .lifetimes
            .parameters
            .entry(declaration.definition)
            .or_default()
            .push(binding);
        self.facts.lifetimes.named.insert(binding, region);
        Ok(Argument::Lifetime(region))
    }
    pub(super) fn library_nominal(&mut self, kind: LibraryType) -> Result<PreparedNominal, Error> {
        let declaration = self
            .report
            .resolution
            .library_types
            .get(kind)
            .cloned()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "unregistered library nominal",
                )
            })?;
        self.build_library_nominal(&declaration)
    }
    fn build_library_nominal(
        &mut self,
        declaration: &LibraryNominal,
    ) -> Result<PreparedNominal, Error> {
        let registered = self.report.resolution.library_types.get(declaration.kind);
        if registered.is_none_or(|registered| {
            registered.definition != declaration.definition
                || registered.parameters != declaration.parameters
                || registered.variants != declaration.variants
        }) {
            return Err(self.error(
                declaration.origin,
                "library nominal differs from its canonical schema",
            ));
        }
        let parameters = self.library_parameters(declaration)?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&parameters)
            .map_err(|e| self.error(declaration.origin, e))?;
        let late =
            declaration.kind == LibraryType::SliceIter && self.facts.types.nominal_sizes_sealed();
        let owner = if late {
            None
        } else {
            Some(self.intern(
                declaration.origin,
                Type::Adt(ItemId(declaration.definition.0), arguments),
            )?)
        };
        if declaration.kind == LibraryType::Ordering {
            self.facts
                .members
                .mark_external_open(ItemId(declaration.definition.0));
        }
        let (kind, repr) = library_representation(declaration.kind);
        let mut variants = Vec::new();
        let mut constructors = Vec::new();
        for ordinal in 0..declaration.variants.len() {
            let (variant, definitions) =
                self.library_variant(declaration, &parameters, ordinal, owner)?;
            variants.push(variant);
            constructors.extend(definitions);
        }
        let item = ItemId(declaration.definition.0);
        if !declaration.kind.is_struct()
            && declaration
                .variants
                .iter()
                .all(|variant| variant.field.is_none())
        {
            self.facts.enum_casts.insert(item);
        }
        self.facts
            .aggregates
            .insert_with_mutation(
                item,
                kind,
                repr,
                &variants,
                if declaration.kind == LibraryType::AtomicU32 {
                    aggregate::SharedMutation::Interior
                } else {
                    aggregate::SharedMutation::Ordinary
                },
            )
            .map_err(|e| self.error(declaration.origin, e))?;
        if declaration.kind == LibraryType::ManuallyDrop {
            self.facts
                .aggregates
                .suppress_field_destruction(item)
                .map_err(|e| self.error(declaration.origin, e))?;
        }
        let owner = if let Some(owner) = owner {
            owner
        } else {
            self.facts
                .types
                .append_sized_leaf(&self.facts.aggregates, item)
                .map_err(|e| self.error(declaration.origin, e))?;
            self.intern(declaration.origin, Type::Adt(item, arguments))?
        };
        self.facts.definitions[declaration.definition.0 as usize] = Some(owner);
        self.library_properties(declaration, item)?;
        self.facts.generics.nominal_contracts.insert(DefId(item.0));
        self.facts.generics.has_nominal_requirements = true;
        Ok(PreparedNominal {
            kind,
            repr,
            constructors,
        })
    }
    fn library_properties(
        &mut self,
        declaration: &LibraryNominal,
        item: ItemId,
    ) -> Result<(), Error> {
        let properties = crate::types::copy::Properties {
            copy: !matches!(
                declaration.kind,
                LibraryType::AtomicU32 | LibraryType::SliceIter
            ),
            clone_: declaration.kind != LibraryType::AtomicU32,
        };
        let selected = if declaration.kind == LibraryType::SliceIter {
            crate::types::copy::Declaration::fields_only_clone()
        } else {
            properties.into()
        };
        self.facts.derives.insert(item, selected);
        if declaration.kind == LibraryType::ComparisonOrdering {
            crate::sema::primitive_methods::register_ordering(self.facts, item)
                .map_err(|e| self.error(declaration.origin, e))?;
            self.facts.equality_derives.insert(
                item,
                crate::types::equality::Properties {
                    partial_eq: true,
                    eq: true,
                },
            );
        }
        Ok(())
    }
    fn library_variant(
        &mut self,
        declaration: &LibraryNominal,
        types: &[Argument],
        ordinal: usize,
        owner: Option<TypeId>,
    ) -> Result<(aggregate::Variant, Vec<DefId>), Error> {
        let variant = declaration.variants[ordinal];
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| self.error(declaration.origin, "too many library variants"))?;
        let fields = self.library_field(declaration, types, variant.field)?;
        let field_types = fields.into_iter().collect::<Vec<_>>();
        let fields = self.library_member_fields(declaration, &field_types)?;
        let shape = if declaration.kind.is_struct() {
            Shape::Record
        } else if fields.is_empty() {
            Shape::Unit
        } else {
            Shape::Tuple
        };
        let id = self
            .facts
            .members
            .insert_metadata(
                ItemId(declaration.definition.0),
                ordinal,
                declaration.origin,
                shape,
                &fields,
            )
            .map_err(|e| self.error(declaration.origin, e))?;
        let list = self
            .facts
            .types
            .intern_list(&field_types)
            .map_err(|e| self.error(declaration.origin, e))?;
        let result = aggregate::Variant {
            fields: list,
            discriminant: declaration
                .kind
                .discriminant(ordinal)
                .ok_or_else(|| self.error(declaration.origin, "invalid library variant ordinal"))?,
        };
        let mut constructors = Vec::new();
        for definition in variant
            .type_definition
            .into_iter()
            .chain(variant.constructor)
        {
            let owner = owner.ok_or_else(|| {
                self.error(
                    declaration.origin,
                    "late library nominal must be constructorless",
                )
            })?;
            self.facts.members.constructors.insert(
                definition,
                crate::sema::members::Constructor {
                    variant: id,
                    kind: if declaration.kind.is_struct() {
                        crate::sema::members::ConstructorKind::ConstFunction
                    } else {
                        crate::sema::members::ConstructorKind::Variant
                    },
                },
            );
            self.facts
                .generics
                .parents
                .insert(definition, declaration.definition);
            constructors.push(definition);
            let ty = if Some(definition) == variant.constructor && !field_types.is_empty() {
                self.library_constructor(
                    declaration.origin,
                    definition,
                    declaration.definition,
                    list,
                    owner,
                )?
            } else {
                owner
            };
            self.facts.definitions[definition.0 as usize] = Some(ty);
        }
        Ok((result, constructors))
    }
    fn library_member_fields(
        &mut self,
        declaration: &LibraryNominal,
        field_types: &[TypeId],
    ) -> Result<Vec<Field>, Error> {
        let name = if declaration.kind.is_struct() {
            self.facts
                .members
                .names
                .intern("value")
                .map(Name::Named)
                .map_err(|e| self.error(declaration.origin, e))?
        } else {
            Name::Index(0)
        };
        let fields = field_types
            .iter()
            .copied()
            .map(|ty| Field {
                origin: crate::sema::members::Origin::Library(declaration.origin),
                name,
                ty,
                access: if declaration.kind.is_struct() {
                    Access::ExternalPrivate
                } else {
                    Access::Public
                },
            })
            .collect::<Vec<_>>();
        Ok(fields)
    }
    fn library_field(
        &mut self,
        declaration: &LibraryNominal,
        types: &[Argument],
        field: Option<LibraryField>,
    ) -> Result<Option<TypeId>, Error> {
        field
            .map(|field| match field {
                LibraryField::U32 => Ok(TypeId::primitive(crate::types::Primitive::U32)),
                LibraryField::Parameter(index) => match types.get(index as usize) {
                    Some(Argument::Type(ty)) => Ok(*ty),
                    _ => Err(self.error(declaration.origin, "invalid library field parameter")),
                },
                LibraryField::SharedSlice => {
                    let [Argument::Lifetime(region), Argument::Type(element)] = types else {
                        return Err(
                            self.error(declaration.origin, "invalid shared slice field schema")
                        );
                    };
                    if declaration.kind != LibraryType::SliceIter {
                        return Err(self.error(
                            declaration.origin,
                            "shared slice field requires canonical Iter",
                        ));
                    }
                    let slice = self.intern(declaration.origin, Type::Slice(*element))?;
                    self.intern(
                        declaration.origin,
                        Type::Reference {
                            mutable: false,
                            region: *region,
                            referent: slice,
                        },
                    )
                }
            })
            .transpose()
    }
    fn library_constructor(
        &mut self,
        source: NodeKey,
        definition: DefId,
        owner: DefId,
        parameters: crate::types::ListId,
        result: TypeId,
    ) -> Result<TypeId, Error> {
        let signature = self.intern(
            source,
            Type::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
            },
        )?;
        self.constructor_template(source, ItemId(definition.0), ItemId(owner.0), signature)?;
        if self
            .facts
            .lifetimes
            .callables
            .get(ItemId(definition.0))
            .is_none()
        {
            self.facts
                .lifetimes
                .callables
                .insert_monomorphic(&self.facts.types, ItemId(definition.0), signature)
                .map_err(|e| self.error(source, e))?;
        }
        self.intern(
            source,
            Type::FunctionItem {
                item: ItemId(definition.0),
                arguments: GenericArgs::EMPTY,
                signature,
            },
        )
    }
}

fn library_representation(library: LibraryType) -> (aggregate::Kind, Repr) {
    let kind = if library.is_struct() {
        aggregate::Kind::Struct
    } else {
        aggregate::Kind::Enum
    };
    let repr = Repr {
        representation: if library == LibraryType::ManuallyDrop {
            Representation::Transparent
        } else {
            Representation::Rust
        },
        tag: library.tag().map(|tag| match tag {
            LibraryTag::U8 => crate::types::Primitive::U8,
            LibraryTag::I8 => crate::types::Primitive::I8,
        }),
        ..Repr::default()
    };
    (kind, repr)
}
