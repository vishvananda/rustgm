//! Equality traits use real canonical owners and ordinary independent shared borrows.
use super::{Checker, Error, ItemId, State, Type};
use crate::resolve::{expansion::Report, model::DefId, unit::NodeKey};
use crate::sema::model::Declarations;
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
            if let Some(declaration) = self.report.resolution.builtin_traits.comparison(kind)
                && !self
                    .report
                    .resolution
                    .builtin_traits
                    .comparison_latent(kind)
            {
                self.comparison_contract(declaration)?;
            }
        }
        Ok(())
    }
    pub(super) fn comparison_family(&mut self, declaration: BuiltinEquality) -> Result<(), Error> {
        if matches!(declaration.kind, Comparison::PartialOrd | Comparison::Ord) {
            for kind in RESULTS {
                let nominal = self
                    .report
                    .resolution
                    .library_types
                    .get(kind)
                    .cloned()
                    .ok_or_else(|| {
                        self.error(declaration.origin, "missing comparison result nominal")
                    })?;
                if !self
                    .facts
                    .generics
                    .nominal_contracts
                    .contains(&nominal.definition)
                {
                    self.library_nominal(&nominal)?;
                }
            }
        }
        for kind in Comparison::ALL {
            if required(declaration.kind, kind) {
                let parent = self
                    .report
                    .resolution
                    .builtin_traits
                    .comparison(kind)
                    .ok_or_else(|| {
                        self.error(declaration.origin, "missing comparison prerequisite")
                    })?;
                if self
                    .facts
                    .associated
                    .comparison(parent.definition)
                    .is_none()
                {
                    self.comparison_contract(parent)?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn comparison_contract(
        &mut self,
        declaration: BuiltinEquality,
    ) -> Result<(), Error> {
        LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .comparison_contract(declaration)?;
        self.states[declaration.definition.0 as usize] = State::Complete;
        self.states[declaration.self_parameter.0 as usize] = State::Complete;
        if let Some(argument) = declaration.argument {
            self.states[argument.0 as usize] = State::Complete;
        }
        if let Some(methods) = declaration.methods {
            for (_, member) in methods.iter() {
                self.states[member.0 as usize] = State::Complete;
            }
        }
        Ok(())
    }
}
/// The exact canonical declaration builder has no source-definition dependencies.
pub(super) struct LibraryContracts<'a> {
    pub report: &'a Report,
    pub facts: &'a mut Declarations,
}
/// Activate only an identity reserved for implicit comparison syntax. A named trait,
/// source implementation or ordinary declaration never enters this route.
pub(in crate::sema) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    owner: DefId,
) -> Result<(), Error> {
    let declaration = report
        .resolution
        .builtin_traits
        .latent_comparison_owner(owner)
        .filter(|declaration| declaration.definition == owner)
        .ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "unregistered latent comparison owner",
            )
        })?;
    if facts.associated.comparison(owner).is_some() {
        return Ok(());
    }
    let mut builder = LibraryContracts { report, facts };
    builder.prepare_results(declaration)?;
    for kind in Comparison::ALL {
        if required(declaration.kind, kind) {
            let parent = builder
                .report
                .resolution
                .builtin_traits
                .comparison(kind)
                .ok_or_else(|| {
                    builder.error(declaration.origin, "missing comparison prerequisite")
                })?;
            if builder
                .facts
                .associated
                .comparison(parent.definition)
                .is_none()
            {
                builder.comparison_contract(parent)?;
            }
        }
    }
    if let Some(contracts) = &mut builder.facts.generics.traits {
        contracts.complete_results();
    }
    Ok(())
}
const RESULTS: [crate::support::library_types::LibraryType; 2] = [
    crate::support::library_types::LibraryType::ComparisonOrdering,
    crate::support::library_types::LibraryType::Option,
];
fn required(owner: Comparison, candidate: Comparison) -> bool {
    candidate == Comparison::PartialEq || candidate == owner || owner == Comparison::Ord
}
impl LibraryContracts<'_> {
    fn prepare_results(&mut self, declaration: BuiltinEquality) -> Result<(), Error> {
        if !matches!(declaration.kind, Comparison::PartialOrd | Comparison::Ord) {
            return Ok(());
        }
        for kind in RESULTS {
            let nominal = self
                .report
                .resolution
                .library_types
                .get(kind)
                .ok_or_else(|| {
                    self.error(declaration.origin, "missing comparison result nominal")
                })?;
            if self
                .facts
                .generics
                .nominal_contracts
                .contains(&nominal.definition)
            {
                continue;
            }
            if !self
                .report
                .resolution
                .builtin_traits
                .comparison_result_latent(kind)
            {
                return Err(self.error(declaration.origin, "comparison result was not completed"));
            }
            self.library_nominal(kind)?;
        }
        Ok(())
    }
    pub(super) fn error(&self, origin: NodeKey, message: impl Into<String>) -> Error {
        Declarations::error(self.report, origin, message)
    }
    pub(super) fn intern(&mut self, origin: NodeKey, kind: Type) -> Result<TypeId, Error> {
        self.facts
            .types
            .intern(kind)
            .map_err(|error| self.error(origin, error))
    }
    pub(super) fn language_parameter(
        &mut self,
        source: NodeKey,
        binding: DefId,
        owner: DefId,
        sized: bool,
    ) -> Result<TypeId, Error> {
        let parameter = self
            .facts
            .types
            .intern_parameter(crate::types::parameters::Parameter {
                owner: ItemId(owner.0),
                index: 0,
                kind: crate::types::parameters::ParameterKind::Type { sized },
            })
            .map_err(|error| self.error(source, error))?;
        let ty = self.intern(source, Type::Parameter(parameter))?;
        self.facts.definitions[binding.0 as usize] = Some(ty);
        Ok(ty)
    }
    pub(super) fn comparison_contract(
        &mut self,
        declaration: BuiltinEquality,
    ) -> Result<(), Error> {
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
            self_bounds: crate::support::hash::HashSet::default(),
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
        // Publish only after the complete schema, generic environment and every
        // method signature are installed. Typed consumers cannot see a prefix.
        self.facts
            .associated
            .declare_comparison(definition, kind, origin, methods);
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
