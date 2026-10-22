//! Collection parameters retain impl lifetime/extent independently of next's borrow.
use super::{
    Argument, BuiltinCollectionDeclaration, BuiltinIteration, Collection, Error, ItemId, RegionId,
    Type, TypeId,
};
use crate::{
    sema::generics::Parameter,
    types::{
        Primitive,
        arguments::ConstKind,
        parameters::{Parameter as Formal, ParameterKind},
    },
};
impl super::super::super::comparisons::LibraryContracts<'_> {
    pub(super) fn collection_parameters(
        &mut self,
        iteration: BuiltinIteration,
        declaration: BuiltinCollectionDeclaration,
    ) -> Result<(TypeId, TypeId, TypeId, TypeId), Error> {
        let source = iteration.origin;
        let lifetime = RegionId::bound(0, 0).map_err(|e| self.error(source, e))?;
        let mut parameters = Vec::new();
        if let Some(binding) = declaration.lifetime {
            parameters.push(Parameter {
                binding,
                anonymous: false,
                argument: Argument::Lifetime(lifetime),
                default: None,
            });
            self.facts
                .lifetimes
                .parameters
                .insert(declaration.definition, vec![binding]);
            self.facts.lifetimes.named.insert(binding, lifetime);
        }
        let id = self
            .facts
            .types
            .intern_parameter(Formal {
                owner: ItemId(declaration.definition.0),
                index: u32::from(declaration.lifetime.is_some()),
                kind: ParameterKind::Type { sized: true },
            })
            .map_err(|e| self.error(source, e))?;
        let parameter = self.intern(source, Type::Parameter(id))?;
        parameters.push(Parameter {
            binding: declaration.parameter,
            anonymous: false,
            argument: Argument::Type(parameter),
            default: None,
        });
        self.facts.definitions[declaration.parameter.0 as usize] = Some(parameter);
        let slice = self.intern(source, Type::Slice(parameter))?;
        let shared = self.intern(
            source,
            Type::Reference {
                referent: parameter,
                region: lifetime,
                mutable: declaration.kind.mutable(),
            },
        )?;
        let iterator_id = self
            .report
            .resolution
            .builtin_traits
            .collections()
            .ok_or_else(|| self.error(source, "missing collection identities"))?
            .iterator(declaration.kind)
            .ok_or_else(|| self.error(source, "missing collection iterator identity"))?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Lifetime(lifetime), Argument::Type(parameter)])
            .map_err(|e| self.error(source, e))?;
        let iterator = self.intern(source, Type::Adt(ItemId(iterator_id.0), arguments))?;
        let subject = match declaration.kind {
            Collection::SliceMethod | Collection::SliceMutableMethod => slice,
            Collection::SliceIterator | Collection::SliceMutableIterator => iterator,
            Collection::SharedSlice | Collection::MutableSlice => self.intern(
                source,
                Type::Reference {
                    referent: slice,
                    region: lifetime,
                    mutable: declaration.kind.mutable(),
                },
            )?,
            Collection::SharedArray | Collection::MutableArray => {
                let array =
                    self.collection_array(iteration, declaration, parameter, &mut parameters)?;
                self.intern(
                    source,
                    Type::Reference {
                        referent: array,
                        region: lifetime,
                        mutable: declaration.kind.mutable(),
                    },
                )?
            }
        };
        self.facts
            .generics
            .parameters
            .insert(declaration.definition, parameters);
        Ok((parameter, subject, shared, iterator))
    }
    fn collection_array(
        &mut self,
        iteration: BuiltinIteration,
        declaration: BuiltinCollectionDeclaration,
        parameter: TypeId,
        parameters: &mut Vec<Parameter>,
    ) -> Result<TypeId, Error> {
        let binding = declaration
            .extent
            .ok_or_else(|| self.error(iteration.origin, "missing collection extent"))?;
        let usize_ = TypeId::primitive(Primitive::Usize);
        let id = self
            .facts
            .types
            .intern_parameter(Formal {
                owner: ItemId(declaration.definition.0),
                index: 2,
                kind: ParameterKind::Const { ty: usize_ },
            })
            .map_err(|e| self.error(iteration.origin, e))?;
        let length = self
            .facts
            .types
            .intern_const(ConstKind::Parameter(id))
            .map_err(|e| self.error(iteration.origin, e))?;
        parameters.push(Parameter {
            binding,
            anonymous: false,
            argument: Argument::Const(length),
            default: None,
        });
        self.facts.generics.constants.insert(binding, length);
        self.facts.definitions[binding.0 as usize] = Some(usize_);
        let array = self.intern(
            iteration.origin,
            Type::ArrayTerm {
                element: parameter,
                length,
            },
        )?;
        Ok(array)
    }
}
