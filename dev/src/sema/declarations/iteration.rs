//! Canonical iteration declarations preserve associated input and output obligations.
mod blanket_methods;
mod blankets;
mod conformance;
use super::{Checker, DefId, Error, ItemId, State, Type};
use crate::{
    resolve::model::{BuiltinIteration, BuiltinIterationDeclaration},
    sema::{
        associated::{Schema, TypeContract},
        generics::{Parameter, Predicate as Written},
        traits::Header,
    },
    support::iteration::Trait,
    types::{
        GenericArgs, RegionId, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        projection,
        traits::Application,
    },
};
type Outputs = Vec<(DefId, Vec<Predicate>)>;
impl Checker<'_> {
    pub(super) fn prepare_iteration_contracts(&mut self) -> Result<(), Error> {
        let Some(canonical) = self.report.resolution.builtin_traits.iteration() else {
            return Ok(());
        };
        self.facts.iteration = Some(Box::new(crate::sema::iteration::Registry::new(canonical)));
        for declaration in canonical.declarations {
            self.iteration_contract(canonical, declaration)?;
        }
        self.prepare_iteration_blankets(canonical)?;
        Ok(())
    }
    fn iteration_contract(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationDeclaration,
    ) -> Result<(), Error> {
        let source = canonical.origin;
        let (subject, application) = self.iteration_trait_owner(canonical, declaration)?;
        let (result, outputs) =
            self.iteration_outputs(canonical, declaration, subject, application)?;
        let receiver = if declaration.kind == Trait::Iterator {
            self.intern(
                source,
                Type::Reference {
                    referent: subject,
                    mutable: true,
                    region: RegionId::bound(0, 0).map_err(|e| self.error(source, e))?,
                },
            )?
        } else {
            subject
        };
        let parameters = self
            .facts
            .types
            .intern_list(&[receiver])
            .map_err(|e| self.error(source, e))?;
        let signature = self.intern(
            source,
            Type::Function {
                parameters,
                result,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
                bound_regions: u32::from(declaration.kind == Trait::Iterator),
            },
        )?;
        self.register_library_method(
            super::comparison_methods::MethodContract {
                origin: source,
                owner: declaration.definition,
                self_parameter: declaration.self_parameter,
                definition: declaration.method,
                signature,
                output: Some(declaration.item),
            },
            subject,
            &[],
        )?;
        let mut results = Vec::new();
        for (member, bounds) in outputs {
            let projection = self.facts.definitions[member.0 as usize]
                .ok_or_else(|| self.error(source, "missing iteration associated declaration"))?;
            self.facts.associated.types.insert(
                member,
                TypeContract {
                    projection,
                    bounds: bounds.iter().copied().map(Into::into).collect(),
                },
            );
            for value in bounds {
                results.push((
                    ItemId(member.0),
                    Written {
                        source,
                        value: value.into(),
                    },
                ));
            }
            self.facts.associated.register_library_type_member(
                declaration.definition,
                member,
                self.report.resolution.definitions[member.0 as usize].name,
            );
        }
        self.facts
            .generics
            .traits
            .get_or_insert_default()
            .declare(
                ItemId(declaration.definition.0),
                Header {
                    self_type: subject,
                    self_bounds: std::collections::HashSet::new(),
                    parameters: Vec::new(),
                    predicates: Vec::new(),
                    results,
                    regions: Vec::new(),
                },
            )
            .map_err(|e| self.error(source, e))?;
        self.states[declaration.definition.0 as usize] = State::Complete;
        Ok(())
    }
    fn iteration_trait_owner(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationDeclaration,
    ) -> Result<(TypeId, crate::types::traits::ApplicationId), Error> {
        let source = canonical.origin;
        let subject = self.language_parameter(
            source,
            declaration.self_parameter,
            declaration.self_parameter,
            false,
        )?;
        self.facts.generics.parameters.insert(
            declaration.self_parameter,
            vec![Parameter {
                binding: declaration.self_parameter,
                anonymous: false,
                argument: Argument::Type(subject),
                default: None,
            }],
        );
        self.facts
            .generics
            .parameters
            .insert(declaration.definition, Vec::new());
        self.facts
            .generics
            .parents
            .insert(declaration.definition, declaration.self_parameter);
        self.facts
            .associated
            .declare_language_trait(
                declaration.definition,
                Schema {
                    self_type: subject,
                    captures: 0,
                    bounds: Vec::new(),
                },
                None,
            )
            .map_err(|e| self.error(source, e))?;
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(declaration.definition.0),
                arguments: GenericArgs::EMPTY,
            })
            .map_err(|e| self.error(source, e))?;
        let own = Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        };
        self.facts
            .generics
            .trait_self_bounds
            .insert(declaration.definition, own);
        self.facts
            .generics
            .predicates
            .entry(declaration.definition)
            .or_default()
            .push(Written {
                source,
                value: own.into(),
            });
        Ok((subject, application))
    }
    fn iteration_outputs(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationDeclaration,
        subject: TypeId,
        application: crate::types::traits::ApplicationId,
    ) -> Result<(TypeId, Outputs), Error> {
        let source = canonical.origin;
        let item = self.iteration_projection(
            canonical,
            declaration,
            declaration.item,
            subject,
            application,
        )?;
        let mut outputs = vec![(
            declaration.item,
            vec![Predicate {
                ty: item,
                bound: Bound::Sized,
            }],
        )];
        let result = if let Some(member) = declaration.into_iter {
            let iterator =
                self.iteration_projection(canonical, declaration, member, subject, application)?;
            let parent = canonical.get(Trait::Iterator);
            let parent_application = self
                .facts
                .types
                .intern_trait_application(Application {
                    item: ItemId(parent.definition.0),
                    arguments: GenericArgs::EMPTY,
                })
                .map_err(|e| self.error(source, e))?;
            let child_item = self.intern(
                source,
                Type::Projection(projection::Application {
                    item: ItemId(parent.item.0),
                    subject: iterator,
                    trait_application: parent_application,
                    arguments: GenericArgs::EMPTY,
                }),
            )?;
            outputs.push((
                member,
                vec![
                    Predicate {
                        ty: iterator,
                        bound: Bound::Sized,
                    },
                    Predicate {
                        ty: iterator,
                        bound: Bound::Trait(parent_application),
                    },
                    Predicate {
                        ty: child_item,
                        bound: Bound::ProjectionEqual(item),
                    },
                ],
            ));
            iterator
        } else {
            let option = self
                .report
                .resolution
                .library_types
                .get(crate::support::library_types::LibraryType::Option)
                .ok_or_else(|| self.error(source, "missing canonical iteration Option"))?
                .definition;
            let arguments = self
                .facts
                .types
                .intern_arguments(&[Argument::Type(item)])
                .map_err(|e| self.error(source, e))?;
            self.intern(source, Type::Adt(ItemId(option.0), arguments))?
        };
        Ok((result, outputs))
    }
    fn iteration_projection(
        &mut self,
        canonical: BuiltinIteration,
        declaration: BuiltinIterationDeclaration,
        member: DefId,
        subject: TypeId,
        application: crate::types::traits::ApplicationId,
    ) -> Result<TypeId, Error> {
        let source = canonical.origin;
        self.facts
            .types
            .declare_projection(
                ItemId(member.0),
                projection::Schema {
                    trait_item: ItemId(declaration.definition.0),
                    sized: true,
                    copies: crate::types::copy::Properties::default(),
                },
            )
            .map_err(|e| self.error(source, e))?;
        let ty = self.intern(
            source,
            Type::Projection(projection::Application {
                item: ItemId(member.0),
                subject,
                trait_application: application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        self.facts.definitions[member.0 as usize] = Some(ty);
        self.facts.generics.parameters.insert(member, Vec::new());
        self.facts
            .generics
            .parents
            .insert(member, declaration.definition);
        self.states[member.0 as usize] = State::Complete;
        Ok(ty)
    }
}
