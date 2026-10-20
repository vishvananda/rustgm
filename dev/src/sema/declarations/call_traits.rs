//! Fn contracts enter the ordinary trait and associated-result machinery as typed metadata.
mod output_binding;
mod syntax;
use super::{Checker, DefId, Error, ItemId, NodeKey, State, Type};
use crate::{
    resolve::model::BuiltinCallable,
    sema::{
        associated::Schema,
        generics::{Parameter, Predicate as Located},
        traits::Header,
    },
    support::call_traits::CallTrait,
    types::{
        GenericArgs, TypeId,
        arguments::Argument,
        parameters::{
            Parameter as Formal, ParameterKind,
            predicates::{Bound, Predicate},
        },
        projection,
        traits::Application,
    },
};

impl Checker<'_> {
    pub(super) fn prepare_call_contracts(&mut self) -> Result<(), Error> {
        // Reverse hierarchy order registers Output before inherited applications.
        for kind in CallTrait::ALL.into_iter().rev() {
            let Some(declaration) = self.report.resolution.builtin_traits.callable(kind) else {
                continue;
            };
            self.call_contract(kind, declaration)?;
        }
        Ok(())
    }

    fn call_contract(
        &mut self,
        kind: CallTrait,
        declaration: BuiltinCallable,
    ) -> Result<(), Error> {
        let BuiltinCallable {
            origin,
            definition,
            output,
            ..
        } = declaration;
        let (self_type, arguments) = self.declare_call_inputs(declaration)?;
        let mut header = Header {
            self_type,
            self_bounds: std::collections::HashSet::new(),
            parameters: self.facts.generics.parameters[&definition].clone(),
            predicates: Vec::new(),
            results: Vec::new(),
            regions: Vec::new(),
        };
        if let Some(parent) = kind.parent() {
            let parent = self
                .report
                .resolution
                .builtin_traits
                .callable(parent)
                .ok_or_else(|| self.error(origin, "missing parent call trait"))?;
            let application = self
                .facts
                .types
                .intern_trait_application(Application {
                    item: ItemId(parent.definition.0),
                    arguments,
                })
                .map_err(|e| self.error(origin, e))?;
            header.self_bounds.insert(origin);
            header.predicates.push(Located {
                source: origin,
                value: Predicate {
                    ty: self_type,
                    bound: Bound::Trait(application),
                }
                .into(),
            });
        }
        if let Some(output) = output {
            let projected = self.call_output(origin, definition, output, self_type, arguments)?;
            header.results.push((
                ItemId(output.0),
                Located {
                    source: origin,
                    value: Predicate {
                        ty: projected,
                        bound: Bound::Sized,
                    }
                    .into(),
                },
            ));
        }
        self.facts
            .generics
            .traits
            .get_or_insert_with(Default::default)
            .declare(ItemId(definition.0), header)
            .map_err(|e| self.error(origin, e))
    }

    /// Register the Self/Args identities and owner environment before building predicates.
    fn declare_call_inputs(
        &mut self,
        declaration: BuiltinCallable,
    ) -> Result<(TypeId, GenericArgs), Error> {
        let BuiltinCallable {
            origin,
            definition,
            self_parameter,
            arguments_parameter,
            output,
        } = declaration;
        let self_type = self.call_parameter(origin, self_parameter, self_parameter, false)?;
        let arguments = self.call_parameter(origin, arguments_parameter, definition, true)?;
        self.facts.generics.parameters.insert(
            definition,
            vec![Parameter {
                binding: arguments_parameter,
                anonymous: false,
                argument: Argument::Type(arguments),
                default: None,
            }],
        );
        self.facts.generics.parameters.insert(
            self_parameter,
            vec![Parameter {
                binding: self_parameter,
                anonymous: false,
                argument: Argument::Type(self_type),
                default: None,
            }],
        );
        self.facts
            .generics
            .parents
            .insert(definition, self_parameter);
        self.facts
            .associated
            .declare_language_trait(
                definition,
                Schema {
                    self_type,
                    captures: 0,
                    bounds: Vec::new(),
                },
                output,
            )
            .map_err(|e| self.error(origin, e))?;
        let arguments = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(arguments)])
            .map_err(|e| self.error(origin, e))?;
        Ok((self_type, arguments))
    }

    fn call_parameter(
        &mut self,
        source: NodeKey,
        binding: DefId,
        owner: DefId,
        sized: bool,
    ) -> Result<TypeId, Error> {
        let parameter = self
            .facts
            .types
            .intern_parameter(Formal {
                owner: ItemId(owner.0),
                index: 0,
                kind: ParameterKind::Type { sized },
            })
            .map_err(|e| self.error(source, e))?;
        let ty = self.intern(source, Type::Parameter(parameter))?;
        self.facts.definitions[binding.0 as usize] = Some(ty);
        self.states[binding.0 as usize] = State::Complete;
        Ok(ty)
    }

    fn call_output(
        &mut self,
        source: NodeKey,
        owner: DefId,
        output: DefId,
        subject: TypeId,
        arguments: GenericArgs,
    ) -> Result<TypeId, Error> {
        self.facts
            .types
            .declare_projection(
                ItemId(output.0),
                projection::Schema {
                    trait_item: ItemId(owner.0),
                    sized: true,
                    copies: crate::types::copy::Properties::default(),
                },
            )
            .map_err(|e| self.error(source, e))?;
        let trait_application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(owner.0),
                arguments,
            })
            .map_err(|e| self.error(source, e))?;
        let projected = self.intern(
            source,
            Type::Projection(projection::Application {
                item: ItemId(output.0),
                subject,
                trait_application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        self.facts.associated.types.insert(
            output,
            crate::sema::associated::TypeContract {
                projection: projected,
                bounds: vec![
                    Predicate {
                        ty: projected,
                        bound: Bound::Sized,
                    }
                    .into(),
                ],
            },
        );
        self.facts.generics.parameters.insert(output, Vec::new());
        self.facts.generics.parents.insert(output, owner);
        self.facts.definitions[output.0 as usize] = Some(projected);
        self.states[output.0 as usize] = State::Complete;
        Ok(projected)
    }
}
