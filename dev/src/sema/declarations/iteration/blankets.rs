//! Real library blanket headers participate in the ordinary candidate inventory.
use super::{Checker, Error, ItemId, State, Type};
use crate::{
    resolve::model::{BuiltinIteration, BuiltinIterationBlanket},
    sema::{
        associated::Schema,
        generics::{Parameter, Predicate as Written},
        iteration::Original,
        traits::implementations::Implementation,
    },
    support::iteration::{Blanket, Trait},
    types::{
        GenericArgs, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            Parameter as Identity, ParameterKind,
            predicates::{Bound, Predicate},
        },
        projection,
        traits::Application,
    },
};
impl Checker<'_> {
    pub(in super::super) fn prepare_iteration_blankets(
        &mut self,
        canonical: BuiltinIteration,
    ) -> Result<(), Error> {
        for declaration in canonical.blankets {
            let original = self.iteration_blanket(canonical, declaration)?;
            self.facts
                .iteration
                .as_deref_mut()
                .ok_or_else(|| Error::at(canonical.origin.file, 0, "missing iteration registry"))?
                .originals[declaration.kind as usize] = Some(original);
        }
        Ok(())
    }
    fn iteration_blanket(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationBlanket,
    ) -> Result<Original, Error> {
        let source = canonical.origin;
        let sized = declaration.kind == Blanket::Identity;
        let (parameter, subject) = self.iteration_impl_parameters(canonical, declaration)?;
        let iterator = canonical.get(Trait::Iterator);
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(iterator.definition.0),
                arguments: GenericArgs::EMPTY,
            })
            .map_err(|e| self.error(source, e))?;
        let condition = Predicate {
            ty: parameter,
            bound: Bound::Trait(application),
        };
        self.facts.generics.predicates.insert(
            declaration.definition,
            vec![Written {
                source,
                value: condition.into(),
            }],
        );
        let item = self.intern(
            source,
            Type::Projection(projection::Application {
                item: ItemId(iterator.item.0),
                subject: parameter,
                trait_application: application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        self.iteration_impl_type(canonical, declaration, declaration.item, item)?;
        if let Some(member) = declaration.into_iter {
            self.iteration_impl_type(canonical, declaration, member, parameter)?;
        }
        let owner = canonical.get(if sized {
            Trait::IntoIterator
        } else {
            Trait::Iterator
        });
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(owner.definition.0),
                arguments: GenericArgs::EMPTY,
            })
            .map_err(|e| self.error(source, e))?;
        let predicate = Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        };
        self.facts.definitions[declaration.definition.0 as usize] = Some(subject);
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| Error::at(source.file, 0, "iteration headers were not reserved"))?
            .prepare_factory(
                &mut self.facts.types,
                &self.facts.associated,
                ItemId(owner.definition.0),
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
        let signature =
            self.iteration_blanket_method(canonical, declaration, parameter, subject, item)?;
        for id in [
            declaration.definition,
            declaration.parameter,
            declaration.method,
        ] {
            self.states[id.0 as usize] = State::Complete;
        }
        let environment = self
            .facts
            .source_parameter_environment(self.report, source, declaration.definition)
            .map_err(|e| self.error(source, e))?;
        self.facts
            .generics
            .environments
            .insert(declaration.definition, std::sync::Arc::new(environment));
        Ok(Original {
            declaration,
            parameter,
            predicate,
            signature,
        })
    }
    fn iteration_impl_parameters(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationBlanket,
    ) -> Result<(TypeId, TypeId), Error> {
        let source = canonical.origin;
        let sized = declaration.kind == Blanket::Identity;
        let id = self
            .facts
            .types
            .intern_parameter(Identity {
                owner: ItemId(declaration.definition.0),
                index: u32::from(!sized),
                kind: ParameterKind::Type { sized },
            })
            .map_err(|e| self.error(source, e))?;
        let parameter = self.intern(source, Type::Parameter(id))?;
        self.facts.definitions[declaration.parameter.0 as usize] = Some(parameter);
        let mut parameters = Vec::new();
        let subject = if let Some(lifetime) = declaration.lifetime {
            let region = RegionId::bound(0, 0).map_err(|e| self.error(source, e))?;
            parameters.push(Parameter {
                binding: lifetime,
                anonymous: false,
                argument: Argument::Lifetime(region),
                default: None,
            });
            self.states[lifetime.0 as usize] = State::Complete;
            self.intern(
                source,
                Type::Reference {
                    referent: parameter,
                    mutable: true,
                    region,
                },
            )?
        } else {
            parameter
        };
        parameters.push(Parameter {
            binding: declaration.parameter,
            anonymous: false,
            argument: Argument::Type(parameter),
            default: None,
        });
        self.facts
            .generics
            .parameters
            .insert(declaration.definition, parameters);
        self.facts
            .associated
            .declare_language_trait(
                declaration.definition,
                Schema {
                    self_type: subject,
                    captures: u32::from(!sized),
                    bounds: Vec::new(),
                },
                None,
            )
            .map_err(|e| self.error(source, e))?;
        Ok((parameter, subject))
    }
    fn iteration_impl_type(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationBlanket,
        member: crate::resolve::model::DefId,
        ty: TypeId,
    ) -> Result<(), Error> {
        self.facts.definitions[member.0 as usize] = Some(ty);
        self.facts
            .generics
            .parents
            .insert(member, declaration.definition);
        self.facts.generics.parameters.insert(member, Vec::new());
        self.facts
            .associated
            .register_library_type_member(declaration.definition, member);
        self.facts
            .associated
            .library_impl_member(declaration.definition, member)
            .map_err(|e| self.error(canonical.origin, e))?;
        self.states[member.0 as usize] = State::Complete;
        Ok(())
    }
}
