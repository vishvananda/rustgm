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
        if kind == Comparison::Eq {
            let parent = self
                .report
                .resolution
                .builtin_traits
                .comparison(Comparison::PartialEq)
                .ok_or_else(|| self.error(origin, "missing PartialEq prerequisite"))?;
            let arguments = self
                .facts
                .types
                .intern_arguments(&[Argument::Type(subject)])
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
            let signature = self.intern(
                origin,
                Type::Function {
                    parameters,
                    result: TypeId::primitive(crate::types::Primitive::Bool),
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                    bound_regions: 2,
                },
            )?;
            for method in Method::ALL {
                self.register_library_method(
                    super::comparison_methods::MethodContract {
                        origin,
                        owner: definition,
                        self_parameter,
                        definition: methods[method as usize],
                        signature,
                        output: None,
                    },
                    subject,
                    &[Argument::Type(right)],
                )?;
            }
        }
        Ok(())
    }
}
