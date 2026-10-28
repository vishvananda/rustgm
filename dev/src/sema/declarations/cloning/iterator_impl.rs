//! Iter's Clone implementation has its own impl lifetime and checked receiver contract.
use super::{Error, ItemId, Type};
use crate::{
    resolve::model::{BuiltinCloneImplementation, BuiltinCloning},
    sema::{
        associated::Schema, cloning::Original, generics::Parameter,
        traits::implementations::Implementation,
    },
    types::{
        GenericArgs, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            Parameter as Formal, ParameterKind,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};
impl super::super::comparisons::LibraryContracts<'_> {
    pub(super) fn iterator_clone_original(
        &mut self,
        canonical: BuiltinCloning,
    ) -> Result<Original, Error> {
        let source = canonical.origin;
        let declaration = canonical.implementation.ok_or_else(|| {
            self.error(source, "missing registered iterator Clone implementation")
        })?;
        let (parameter, subject) = self.iterator_clone_parameters(canonical, declaration)?;
        self.facts
            .associated
            .declare_language_trait(
                declaration.definition,
                Schema {
                    self_type: subject,
                    captures: 1,
                    bounds: Vec::new(),
                },
                None,
            )
            .map_err(|e| self.error(source, e))?;
        self.facts.definitions[declaration.definition.0 as usize] = Some(subject);
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(canonical.definition.0),
                arguments: GenericArgs::EMPTY,
            })
            .map_err(|e| self.error(source, e))?;
        let predicate = Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        };
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::at(source.file, 0, "missing Clone header inventory"))?
            .prepare_factory(
                &mut self.facts.types,
                &self.facts.associated,
                ItemId(canonical.definition.0),
                Implementation {
                    owner: declaration.definition,
                    source: Some(source),
                    predicate,
                    methods: Vec::new(),
                    types: Vec::new(),
                    constants: Vec::new(),
                },
            )
            .map_err(|e| self.error(source, e))?;
        self.facts
            .source_complete_environment(self.report, source, declaration.definition)
            .map_err(|e| self.error(source, e))?;
        let signature = self.iterator_clone_method(canonical, declaration, subject)?;
        Ok(Original {
            declaration,
            parameter,
            subject,
            signature,
            predicate,
        })
    }
    fn iterator_clone_method(
        &mut self,
        canonical: BuiltinCloning,
        declaration: BuiltinCloneImplementation,
        subject: TypeId,
    ) -> Result<TypeId, Error> {
        let source = canonical.origin;
        let signature = self.clone_signature(source, subject, 1, 2)?;
        let callable = self.intern(
            source,
            Type::FunctionItem {
                item: ItemId(declaration.method.0),
                signature,
                arguments: GenericArgs::EMPTY,
            },
        )?;
        let Type::Function { parameters, .. } = self
            .facts
            .types
            .kind(signature)
            .ok_or_else(|| self.error(source, "missing iterator Clone signature"))?
        else {
            return Err(self.error(source, "iterator Clone signature is not a function"));
        };
        let receiver = self
            .facts
            .types
            .list(parameters)
            .and_then(|p| p.first())
            .copied()
            .ok_or_else(|| self.error(source, "missing iterator Clone receiver"))?;
        self.publish_library_method(
            super::super::comparison_methods::MethodContract {
                origin: source,
                owner: declaration.definition,
                self_parameter: declaration.parameter,
                definition: declaration.method,
                signature,
                output: None,
            },
            callable,
            receiver,
            2,
        )?;
        self.facts
            .associated
            .library_impl_member(declaration.definition, declaration.method)
            .map_err(|e| self.error(source, e))?;
        let scheme = crate::sema::lifetimes::FunctionRegions {
            total: 2,
            captures: 1,
            early: Vec::new(),
            late: vec![1],
            has_declared_late: false,
            bounds: Vec::new(),
        };
        self.callable_template(source, declaration.method, signature, &scheme)?;
        self.facts
            .lifetimes
            .functions
            .insert(declaration.method, scheme);
        Ok(signature)
    }
    fn iterator_clone_parameters(
        &mut self,
        canonical: BuiltinCloning,
        declaration: BuiltinCloneImplementation,
    ) -> Result<(TypeId, TypeId), Error> {
        let source = canonical.origin;
        let lifetime = RegionId::bound(0, 0).map_err(|e| self.error(source, e))?;
        let id = self
            .facts
            .types
            .intern_parameter(Formal {
                owner: ItemId(declaration.definition.0),
                index: 1,
                kind: ParameterKind::Type { sized: true },
            })
            .map_err(|e| self.error(source, e))?;
        let parameter = self.intern(source, Type::Parameter(id))?;
        self.facts.definitions[declaration.parameter.0 as usize] = Some(parameter);
        self.facts
            .lifetimes
            .parameters
            .insert(declaration.definition, vec![declaration.lifetime]);
        self.facts
            .lifetimes
            .named
            .insert(declaration.lifetime, lifetime);
        self.facts.generics.parameters.insert(
            declaration.definition,
            vec![
                Parameter {
                    binding: declaration.lifetime,
                    anonymous: false,
                    argument: Argument::Lifetime(lifetime),
                    default: None,
                },
                Parameter {
                    binding: declaration.parameter,
                    anonymous: false,
                    argument: Argument::Type(parameter),
                    default: None,
                },
            ],
        );
        let iterator = self
            .report
            .resolution
            .builtin_traits
            .collections()
            .ok_or_else(|| self.error(source, "missing canonical iterator identity"))?
            .iterator;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Lifetime(lifetime), Argument::Type(parameter)])
            .map_err(|e| self.error(source, e))?;
        let subject = self.intern(source, Type::Adt(ItemId(iterator.0), arguments))?;
        Ok((parameter, subject))
    }
}
