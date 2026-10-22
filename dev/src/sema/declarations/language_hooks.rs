//! Operator metadata enters ordinary trait conformance and selected-call dispatch.
use super::{Checker, DefId, Error, ItemId, NodeKey, State, Type};
use crate::{
    resolve::model::{BuiltinLanguage, BuiltinLanguageOutput},
    sema::{associated::Schema, generics::Parameter, traits::Header},
    support::language_hooks::Hook,
    types::{
        GenericArgs, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
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
    pub(super) fn language_parameter(
        &mut self,
        source: NodeKey,
        binding: DefId,
        owner: DefId,
        sized: bool,
    ) -> Result<TypeId, Error> {
        let ty = super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .language_parameter(source, binding, owner, sized)?;
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
        self.language_self_bound(declaration, subject, arguments)?;
        let (receiver, signature) =
            self.language_signature(declaration, subject, projected, &inputs)?;
        self.language_method(declaration, subject, receiver, signature, &inputs)
    }
    fn language_self_bound(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        arguments: GenericArgs,
    ) -> Result<(), Error> {
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(declaration.definition.0),
                arguments,
            })
            .map_err(|e| self.error(declaration.origin, e))?;
        let value = Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        };
        self.facts
            .generics
            .trait_self_bounds
            .insert(declaration.definition, value);
        self.facts
            .generics
            .predicates
            .entry(declaration.definition)
            .or_default()
            .push(crate::sema::generics::Predicate {
                source: declaration.origin,
                value: value.into(),
            });
        Ok(())
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
                default: (!kind.indexing()).then_some(Argument::Type(subject)),
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
                declaration.output.declared(),
            )
            .map_err(|e| self.error(origin, e))?;
        self.facts
            .associated
            .declare_operation(definition, kind, declaration.method);
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
        let Some(member) = output.definition() else {
            self.language_header(declaration, subject, parameters, None, Vec::new())?;
            return Ok(TypeId::UNIT);
        };
        let owner = output
            .owner(definition)
            .ok_or_else(|| self.error(origin, "missing language output owner"))?;
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(owner.0),
                arguments,
            })
            .map_err(|e| self.error(origin, e))?;
        if output.declared().is_some() {
            self.facts
                .types
                .declare_projection(
                    ItemId(member.0),
                    projection::Schema {
                        trait_item: ItemId(definition.0),
                        sized: !kind.indexing(),
                        copies: crate::types::copy::Properties::default(),
                    },
                )
                .map_err(|e| self.error(origin, e))?;
        }
        let projected = self.intern(
            origin,
            Type::Projection(projection::Application {
                item: ItemId(member.0),
                subject,
                trait_application: application,
                arguments: GenericArgs::EMPTY,
            }),
        )?;
        let bounds = if kind.indexing() {
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
        if output.declared().is_some() {
            self.facts.associated.types.insert(
                member,
                crate::sema::associated::TypeContract {
                    projection: projected,
                    bounds: bounds.clone(),
                },
            );
            self.facts.definitions[member.0 as usize] = Some(projected);
            self.states[member.0 as usize] = State::Complete;
            self.facts.generics.parameters.insert(member, Vec::new());
            self.facts.generics.parents.insert(member, definition);
        }
        self.language_header(declaration, subject, parameters, Some(application), bounds)?;
        Ok(projected)
    }
    fn language_header(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        parameters: Vec<Parameter>,
        application: Option<crate::types::traits::ApplicationId>,
        bounds: Vec<crate::types::parameters::clauses::Clause>,
    ) -> Result<(), Error> {
        let BuiltinLanguage {
            origin,
            definition,
            output,
            ..
        } = declaration;
        let mut header = Header {
            self_type: subject,
            self_bounds: std::collections::HashSet::new(),
            parameters,
            predicates: Vec::new(),
            results: bounds
                .into_iter()
                .filter_map(|value| {
                    output.definition().map(|member| {
                        (
                            ItemId(member.0),
                            crate::sema::generics::Predicate {
                                source: origin,
                                value,
                            },
                        )
                    })
                })
                .collect(),
            regions: Vec::new(),
        };
        if matches!(output, BuiltinLanguageOutput::Inherited { .. })
            && let Some(application) = application
        {
            header.self_bounds.insert(origin);
            header.predicates.push(crate::sema::generics::Predicate {
                source: origin,
                value: Predicate {
                    ty: subject,
                    bound: Bound::Trait(application),
                }
                .into(),
            });
        }
        self.facts
            .generics
            .traits
            .get_or_insert_with(Default::default)
            .declare(ItemId(definition.0), header)
            .map_err(|e| self.error(origin, e))?;
        Ok(())
    }
    fn language_signature(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        projected: TypeId,
        inputs: &[Argument],
    ) -> Result<(TypeId, TypeId), Error> {
        let BuiltinLanguage { origin, kind, .. } = declaration;
        let (receiver, result) = if kind.borrowed() {
            let region = crate::types::RegionId::bound(0, 0).map_err(|e| self.error(origin, e))?;
            (
                self.intern(
                    origin,
                    Type::Reference {
                        referent: subject,
                        mutable: kind != Hook::Index,
                        region,
                    },
                )?,
                if kind.indexing() {
                    self.intern(
                        origin,
                        Type::Reference {
                            referent: projected,
                            mutable: kind == Hook::IndexMut,
                            region,
                        },
                    )?
                } else {
                    projected
                },
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
                convention: crate::types::callable::Convention::RUST,
                bound_regions: u32::from(kind.borrowed()),
            },
        )?;
        Ok((receiver, signature))
    }
    fn language_method(
        &mut self,
        declaration: BuiltinLanguage,
        subject: TypeId,
        _receiver: TypeId,
        signature: TypeId,
        inputs: &[Argument],
    ) -> Result<(), Error> {
        let BuiltinLanguage {
            origin,
            definition,
            self_parameter,
            output,
            method,
            ..
        } = declaration;
        self.register_library_method(
            super::comparison_methods::MethodContract {
                origin,
                owner: definition,
                self_parameter,
                definition: method,
                signature,
                output: output.declared(),
            },
            subject,
            inputs,
        )
    }
}
