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
    support::library_types::{LibraryField, LibraryType},
    syntax::item::Shape,
    types::{
        GenericArgs, TypeId,
        aggregate::{self, Repr, Representation},
        arguments::Argument,
        parameters::{Parameter as Formal, ParameterKind},
    },
};
impl Checker<'_> {
    pub(super) fn prepare_library_types(&mut self) -> Result<(), Error> {
        for kind in LibraryType::ALL {
            if let Some(declaration) = self.report.resolution.library_types.get(kind).cloned() {
                self.library_nominal(&declaration)?;
            }
        }
        Ok(())
    }
    fn library_parameters(&mut self, declaration: &LibraryNominal) -> Result<Vec<TypeId>, Error> {
        let mut parameters = Vec::new();
        let mut types = Vec::new();
        for (index, &binding) in declaration.parameters.iter().enumerate() {
            let parameter = self
                .facts
                .types
                .intern_parameter(Formal {
                    owner: ItemId(declaration.definition.0),
                    index: u32::try_from(index).map_err(|_| {
                        self.error(declaration.origin, "too many library parameters")
                    })?,
                    kind: ParameterKind::Type {
                        sized: declaration.kind != LibraryType::ManuallyDrop,
                    },
                })
                .map_err(|e| self.error(declaration.origin, e))?;
            let ty = self.intern(declaration.origin, Type::Parameter(parameter))?;
            self.facts.definitions[binding.0 as usize] = Some(ty);
            self.states[binding.0 as usize] = State::Complete;
            types.push(ty);
            parameters.push(Parameter {
                binding,
                anonymous: false,
                argument: Argument::Type(ty),
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
        Ok(types)
    }
    fn library_nominal(&mut self, declaration: &LibraryNominal) -> Result<(), Error> {
        let types = self.library_parameters(declaration)?;
        let arguments = self
            .facts
            .types
            .intern_arguments(
                &types
                    .iter()
                    .copied()
                    .map(Argument::Type)
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| self.error(declaration.origin, e))?;
        let owner = self.intern(
            declaration.origin,
            Type::Adt(ItemId(declaration.definition.0), arguments),
        )?;
        self.facts.definitions[declaration.definition.0 as usize] = Some(owner);
        self.states[declaration.definition.0 as usize] = State::Complete;
        if declaration.kind == LibraryType::Ordering {
            self.facts
                .members
                .mark_external_open(ItemId(declaration.definition.0));
        }
        let kind = if declaration.kind.is_struct() {
            aggregate::Kind::Struct
        } else {
            aggregate::Kind::Enum
        };
        let repr = Repr {
            representation: if declaration.kind == LibraryType::ManuallyDrop {
                Representation::Transparent
            } else {
                Representation::Rust
            },
            tag: (declaration.kind == LibraryType::Ordering).then_some(crate::types::Primitive::U8),
            ..Repr::default()
        };
        let mut variants = Vec::new();
        let mut constructors = Vec::new();
        for ordinal in 0..declaration.variants.len() {
            let (variant, definitions) =
                self.library_variant(declaration, &types, ordinal, owner)?;
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
        self.facts.derives.insert(
            item,
            crate::types::copy::Properties {
                copy: declaration.kind != LibraryType::AtomicU32,
                clone_: declaration.kind != LibraryType::AtomicU32,
            },
        );
        self.register_library_nominal(item, declaration.origin, kind, repr, &constructors)
    }
    fn library_variant(
        &mut self,
        declaration: &LibraryNominal,
        types: &[TypeId],
        ordinal: usize,
        owner: TypeId,
    ) -> Result<(aggregate::Variant, Vec<DefId>), Error> {
        let variant = declaration.variants[ordinal];
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| self.error(declaration.origin, "too many library variants"))?;
        let fields = self.library_field(declaration, types, variant.field)?;
        let field_types = fields.into_iter().collect::<Vec<_>>();
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
            discriminant: u128::from(ordinal),
        };
        let mut constructors = Vec::new();
        for definition in variant
            .type_definition
            .into_iter()
            .chain(std::iter::once(variant.constructor))
        {
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
            self.states[definition.0 as usize] = State::Complete;
            constructors.push(definition);
            let ty = if definition == variant.constructor && !field_types.is_empty() {
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
    fn library_field(
        &self,
        declaration: &LibraryNominal,
        types: &[TypeId],
        field: Option<LibraryField>,
    ) -> Result<Option<TypeId>, Error> {
        field
            .map(|field| match field {
                LibraryField::U32 => Ok(TypeId::primitive(crate::types::Primitive::U32)),
                LibraryField::Parameter(index) => {
                    types.get(index as usize).copied().ok_or_else(|| {
                        self.error(declaration.origin, "invalid library field parameter")
                    })
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
