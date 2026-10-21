//! Equality traits use real canonical owners and ordinary independent shared borrows.
use super::{Checker, Error, ItemId, State, Type};
use crate::{
    resolve::model::BuiltinEquality,
    sema::{
        associated::Schema,
        generics::{Parameter, Predicate as Written},
        traits::Header,
    },
    support::comparisons::{Comparison, Method},
    types::{
        RegionId, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
impl Checker<'_> {
    pub(super) fn prepare_comparison_contracts(&mut self) -> Result<(), Error> {
        for kind in Comparison::ALL {
            if let Some(declaration) = self.report.resolution.builtin_traits.comparison(kind) {
                self.comparison_contract(declaration)?;
            }
        }
        Ok(())
    }
    fn comparison_contract(&mut self, declaration: BuiltinEquality) -> Result<(), Error> {
        let BuiltinEquality {
            origin,
            kind,
            definition,
            methods,
            ..
        } = declaration;
        let (subject, parameters) = self.comparison_inputs(declaration)?;
        self.facts
            .associated
            .declare_language_trait(
                definition,
                Schema {
                    self_type: subject,
                    captures: 0,
                    bounds: Vec::new(),
                },
                None,
            )
            .map_err(|e| self.error(origin, e))?;
        self.facts
            .associated
            .declare_comparison(definition, kind, origin, methods);
        let arguments = self
            .facts
            .types
            .intern_arguments(&parameters.iter().map(|p| p.argument).collect::<Vec<_>>())
            .map_err(|e| self.error(origin, e))?;
        let app = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(definition.0),
                arguments,
            })
            .map_err(|e| self.error(origin, e))?;
        let own = Predicate {
            ty: subject,
            bound: Bound::Trait(app),
        };
        self.facts
            .generics
            .trait_self_bounds
            .insert(definition, own);
        self.facts
            .generics
            .predicates
            .entry(definition)
            .or_default()
            .push(Written {
                source: origin,
                value: own.into(),
            });
        let mut header = Header {
            self_type: subject,
            self_bounds: std::collections::HashSet::new(),
            parameters,
            predicates: Vec::new(),
            results: Vec::new(),
            regions: Vec::new(),
        };
        self.comparison_parents(declaration, subject, &mut header)?;
        self.facts
            .generics
            .traits
            .get_or_insert_default()
            .declare(ItemId(definition.0), header)
            .map_err(|e| self.error(origin, e))?;
        self.comparison_methods(declaration, subject)?;
        self.states[definition.0 as usize] = State::Complete;
        Ok(())
    }
    fn comparison_inputs(
        &mut self,
        declaration: BuiltinEquality,
    ) -> Result<(TypeId, Vec<Parameter>), Error> {
        let BuiltinEquality {
            origin,
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
        let parameters = if let Some(binding) = argument {
            let ty = self.language_parameter(origin, binding, definition, false)?;
            vec![Parameter {
                binding,
                anonymous: false,
                argument: Argument::Type(ty),
                default: Some(Argument::Type(subject)),
            }]
        } else {
            Vec::new()
        };
        self.facts
            .generics
            .parameters
            .insert(definition, parameters.clone());
        self.facts
            .generics
            .parents
            .insert(definition, self_parameter);
        Ok((subject, parameters))
    }
    fn comparison_methods(
        &mut self,
        declaration: BuiltinEquality,
        subject: TypeId,
    ) -> Result<(), Error> {
        let BuiltinEquality {
            origin,
            definition,
            self_parameter,
            methods,
            ..
        } = declaration;
        if let Some(methods) = methods {
            let right = if declaration.kind.has_rhs() {
                let Some(Argument::Type(right)) = self
                    .facts
                    .generics
                    .parameters
                    .get(&definition)
                    .and_then(|p| p.first())
                    .map(|p| p.argument)
                else {
                    return Err(self.error(origin, "missing comparison RHS"));
                };
                right
            } else {
                subject
            };
            let mut parameters = Vec::new();
            for (slot, referent) in [subject, right].into_iter().enumerate() {
                let region = RegionId::bound(
                    0,
                    u16::try_from(slot)
                        .map_err(|_| self.error(origin, "too many comparison slots"))?,
                )
                .map_err(|e| self.error(origin, e))?;
                parameters.push(self.intern(
                    origin,
                    Type::Reference {
                        referent,
                        mutable: false,
                        region,
                    },
                )?);
            }
            let parameters = self
                .facts
                .types
                .intern_list(&parameters)
                .map_err(|e| self.error(origin, e))?;
            for (method, member) in methods.iter() {
                let signature = self.comparison_signature(origin, method, subject, parameters)?;
                let arguments = if declaration.kind.has_rhs() {
                    vec![Argument::Type(right)]
                } else {
                    Vec::new()
                };
                self.register_library_method(
                    super::comparison_methods::MethodContract {
                        origin,
                        owner: definition,
                        self_parameter,
                        definition: member,
                        signature,
                        output: None,
                    },
                    subject,
                    &arguments,
                )?;
                if method.owned() {
                    self.facts
                        .generics
                        .predicates
                        .entry(member)
                        .or_default()
                        .push(Written {
                            source: origin,
                            value: Predicate {
                                ty: subject,
                                bound: Bound::Sized,
                            }
                            .into(),
                        });
                }
            }
        }
        Ok(())
    }
    fn comparison_parents(
        &mut self,
        declaration: BuiltinEquality,
        subject: TypeId,
        header: &mut Header,
    ) -> Result<(), Error> {
        let origin = declaration.origin;
        let kind = declaration.kind;
        let right = header.parameters.first().map_or(subject, |parameter| {
            if let Argument::Type(ty) = parameter.argument {
                ty
            } else {
                subject
            }
        });
        for &parent_kind in kind.parents() {
            let parent = self
                .report
                .resolution
                .builtin_traits
                .comparison(parent_kind)
                .ok_or_else(|| self.error(origin, "missing comparison prerequisite"))?;
            let args = if parent_kind.has_rhs() {
                vec![Argument::Type(right)]
            } else {
                Vec::new()
            };
            let arguments = self
                .facts
                .types
                .intern_arguments(&args)
                .map_err(|e| self.error(origin, e))?;
            let app = self
                .facts
                .types
                .intern_trait_application(Application {
                    item: ItemId(parent.definition.0),
                    arguments,
                })
                .map_err(|e| self.error(origin, e))?;
            header.self_bounds.insert(origin);
            header.predicates.push(Written {
                source: origin,
                value: Predicate {
                    ty: subject,
                    bound: Bound::Trait(app),
                }
                .into(),
            });
        }
        Ok(())
    }
    fn comparison_signature(
        &mut self,
        origin: crate::resolve::unit::NodeKey,
        method: Method,
        subject: TypeId,
        parameters: crate::types::ListId,
    ) -> Result<TypeId, Error> {
        let result = if method.owned() {
            subject
        } else {
            self.comparison_result(origin, method)?
        };
        let inputs = if method.owned() {
            self.facts
                .types
                .intern_list(&vec![subject; if method == Method::Clamp { 3 } else { 2 }])
                .map_err(|e| self.error(origin, e))?
        } else {
            parameters
        };
        self.intern(
            origin,
            Type::Function {
                parameters: inputs,
                result,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
                bound_regions: if method.owned() { 0 } else { 2 },
            },
        )
    }
    fn comparison_result(
        &mut self,
        origin: crate::resolve::unit::NodeKey,
        method: Method,
    ) -> Result<TypeId, Error> {
        let result = match method {
            Method::PartialCmp | Method::Cmp => {
                let nominal = self
                    .report
                    .resolution
                    .library_types
                    .get(crate::support::library_types::LibraryType::ComparisonOrdering)
                    .ok_or_else(|| self.error(origin, "missing canonical comparison Ordering"))?;
                let ordering =
                    self.facts.definitions[nominal.definition.0 as usize].ok_or_else(|| {
                        self.error(origin, "incomplete canonical comparison Ordering")
                    })?;
                if method == Method::Cmp {
                    ordering
                } else {
                    let option = self
                        .report
                        .resolution
                        .library_types
                        .get(crate::support::library_types::LibraryType::Option)
                        .ok_or_else(|| self.error(origin, "missing canonical comparison Option"))?;
                    let arguments = self
                        .facts
                        .types
                        .intern_arguments(&[Argument::Type(ordering)])
                        .map_err(|e| self.error(origin, e))?;
                    self.intern(origin, Type::Adt(ItemId(option.definition.0), arguments))?
                }
            }
            _ => TypeId::primitive(crate::types::Primitive::Bool),
        };
        Ok(result)
    }
}
