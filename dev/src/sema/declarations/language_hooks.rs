//! Operator metadata enters ordinary trait conformance and selected-call dispatch.
use super::{Checker, DefId, Error, ItemId, NodeKey, State, Type};
use crate::{
    resolve::model::BuiltinLanguage,
    sema::{associated::Schema, generics::Parameter, model::Function, traits::Header},
    support::language_hooks::Hook,
    syntax::ast::List,
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
    pub(super) fn prepare_language_contracts(&mut self) -> Result<(), Error> {
        for kind in Hook::ALL {
            if let Some(declaration) = self.report.resolution.builtin_traits.language(kind) {
                self.language_contract(declaration)?;
            }
        }
        Ok(())
    }
    fn language_parameter(
        &mut self,
        source: NodeKey,
        binding: DefId,
        owner: DefId,
        sized: bool,
    ) -> Result<TypeId, Error> {
        let id = self
            .facts
            .types
            .intern_parameter(Formal {
                owner: ItemId(owner.0),
                index: 0,
                kind: ParameterKind::Type { sized },
            })
            .map_err(|e| self.error(source, e))?;
        let ty = self.intern(source, Type::Parameter(id))?;
        self.facts.definitions[binding.0 as usize] = Some(ty);
        self.states[binding.0 as usize] = State::Complete;
        Ok(ty)
    }
    fn language_contract(&mut self, declaration: BuiltinLanguage) -> Result<(), Error> {
        let (subject, parameters) = self.language_inputs(declaration)?;
        let inputs = parameters.iter().map(|p| p.argument).collect::<Vec<_>>();
        let arguments = self
            .facts
            .types
            .intern_arguments(&inputs)
            .map_err(|e| self.error(declaration.origin, e))?;
        let projected = self.language_output(declaration, subject, arguments, parameters)?;
        let (receiver, signature) =
            self.language_signature(declaration, subject, projected, &inputs)?;
        self.language_method(declaration, subject, receiver, signature, &inputs)
    }
    fn language_inputs(
        &mut self,
        declaration: BuiltinLanguage,
    ) -> Result<(TypeId, Vec<Parameter>), Error> {
        let BuiltinLanguage {
            origin,
            kind,
            definition,
            self_parameter,
            argument,
            ..
        } = declaration;
        let subject = self.language_parameter(origin, self_parameter, self_parameter, false)?;
        self.facts.generics.parameters.insert(
            self_parameter,
            vec![Parameter {
                binding: self_parameter,
                anonymous: false,
                argument: Argument::Type(subject),
                default: None,
            }],
        );
        let mut parameters = Vec::new();
        if let Some(binding) = argument {
            let ty = self.language_parameter(origin, binding, definition, true)?;
            parameters.push(Parameter {
                binding,
                anonymous: false,
                argument: Argument::Type(ty),
                default: (kind != Hook::Index).then_some(Argument::Type(subject)),
            });
        }
        self.facts
            .generics
            .parameters
            .insert(definition, parameters.clone());
        self.facts
            .generics
            .parents
            .insert(definition, self_parameter);
        self.facts
            .associated
            .declare_language_trait(
                definition,
                Schema {
                    self_type: subject,
                    captures: 0,
                    bounds: Vec::new(),
                },
                Some(declaration.output),
            )
            .map_err(|e| self.error(origin, e))?;
        Ok((subject, parameters))
    }
    fn language_output(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        arguments: GenericArgs,
        parameters: Vec<Parameter>,
    ) -> Result<TypeId, Error> {
        let BuiltinLanguage {
            origin,
            kind,
            definition,
            output,
            ..
        } = declaration;
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(definition.0),
                arguments,
            })
            .map_err(|e| self.error(origin, e))?;
        self.facts
            .types
            .declare_projection(
                ItemId(output.0),
                projection::Schema {
                    trait_item: ItemId(definition.0),
                    sized: kind != Hook::Index,
                    copies: crate::types::copy::Properties::default(),
                },
            )
            .map_err(|e| self.error(origin, e))?;
        let projected = self.intern(
            origin,
            Type::Projection(projection::Application {
                item: ItemId(output.0),
                subject,
                trait_application: application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        let bounds = if kind == Hook::Index {
            Vec::new()
        } else {
            vec![
                Predicate {
                    ty: projected,
                    bound: Bound::Sized,
                }
                .into(),
            ]
        };
        self.facts.associated.types.insert(
            output,
            crate::sema::associated::TypeContract {
                projection: projected,
                bounds: bounds.clone(),
            },
        );
        self.facts.definitions[output.0 as usize] = Some(projected);
        self.states[output.0 as usize] = State::Complete;
        self.facts.generics.parameters.insert(output, Vec::new());
        self.facts.generics.parents.insert(output, definition);
        self.facts
            .generics
            .traits
            .get_or_insert_with(Default::default)
            .declare(
                ItemId(definition.0),
                Header {
                    self_type: subject,
                    self_bounds: std::collections::HashSet::new(),
                    parameters,
                    predicates: Vec::new(),
                    results: bounds
                        .into_iter()
                        .map(|value| {
                            (
                                ItemId(output.0),
                                crate::sema::generics::Predicate {
                                    source: origin,
                                    value,
                                },
                            )
                        })
                        .collect(),
                    regions: Vec::new(),
                },
            )
            .map_err(|e| self.error(origin, e))?;
        Ok(projected)
    }
    fn language_signature(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        projected: TypeId,
        inputs: &[Argument],
    ) -> Result<(TypeId, TypeId), Error> {
        let BuiltinLanguage { origin, kind, .. } = declaration;
        let (receiver, result) = if kind == Hook::Index {
            let region = crate::types::RegionId::bound(0, 0).map_err(|e| self.error(origin, e))?;
            (
                self.intern(
                    origin,
                    Type::Reference {
                        referent: subject,
                        mutable: false,
                        region,
                    },
                )?,
                self.intern(
                    origin,
                    Type::Reference {
                        referent: projected,
                        mutable: false,
                        region,
                    },
                )?,
            )
        } else {
            (subject, projected)
        };
        let mut values = vec![receiver];
        if let Some(Argument::Type(ty)) = inputs.first() {
            values.push(*ty);
        }
        let parameters = self
            .facts
            .types
            .intern_list(&values)
            .map_err(|e| self.error(origin, e))?;
        let signature = self.intern(
            origin,
            Type::Function {
                parameters,
                result,
                unsafe_: false,
                bound_regions: u32::from(kind == Hook::Index),
            },
        )?;
        Ok((receiver, signature))
    }
    fn language_method(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        receiver: TypeId,
        signature: TypeId,
        inputs: &[Argument],
    ) -> Result<(), Error> {
        let BuiltinLanguage {
            origin,
            kind,
            definition,
            self_parameter,
            output,
            method,
            ..
        } = declaration;
        let callable = self.intern(
            origin,
            Type::FunctionItem {
                item: ItemId(method.0),
                signature,
                arguments: GenericArgs::EMPTY,
            },
        )?;
        let generic_parameters = std::iter::once(subject)
            .chain(inputs.iter().filter_map(|argument| {
                if let Argument::Type(ty) = argument {
                    Some(*ty)
                } else {
                    None
                }
            }))
            .map(|ty| crate::sema::generics::parameter_id(&self.facts.types, Argument::Type(ty)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| self.error(origin, e))?;
        self.facts
            .lifetimes
            .callables
            .insert_scoped(
                &self.facts.types,
                &[
                    ItemId(self_parameter.0),
                    ItemId(definition.0),
                    ItemId(method.0),
                ],
                signature,
                0,
                &[],
                &generic_parameters,
            )
            .map_err(|e| self.error(origin, e))?;
        self.facts.definitions[method.0 as usize] = Some(callable);
        self.states[method.0 as usize] = State::Complete;
        self.facts.generics.parameters.insert(method, Vec::new());
        self.facts.generics.parents.insert(method, definition);
        self.facts
            .methods
            .receivers
            .insert(method, (receiver, u32::from(kind == Hook::Index)));
        self.facts.lifetimes.functions.insert(
            method,
            crate::sema::lifetimes::FunctionRegions {
                total: u32::from(kind == Hook::Index),
                captures: 0,
                early: Vec::new(),
                late: if kind == Hook::Index {
                    vec![0]
                } else {
                    Vec::new()
                },
                has_declared_late: false,
                bounds: Vec::new(),
            },
        );
        self.facts.associated.register_language_members(
            definition,
            origin,
            Function {
                definition: method,
                node: origin,
                signature,
                parameters: List { start: 0, end: 0 },
                body: None,
            },
            (
                output,
                self.report.resolution.definitions[output.0 as usize].name,
            ),
        );
        Ok(())
    }
}
