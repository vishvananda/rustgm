//! Clone's declared member uses its real identity and independent shared borrow binder.
mod activation;
mod conformance;
mod iterator_impl;
use super::{Checker, Error, ItemId, State, Type};
use crate::{
    resolve::model::BuiltinCloning,
    sema::{generics::Predicate as Written, traits::Header},
    types::{
        RegionId, TypeId,
        parameters::predicates::{Bound, Predicate},
    },
};
pub(in crate::sema) use activation::{owns, prepare, prepare_requested, request_member};
impl Checker<'_> {
    pub(super) fn prepare_clone_contract(&mut self) -> Result<(), Error> {
        let Some(declaration) = self.report.resolution.builtin_traits.cloning() else {
            return Ok(());
        };
        super::comparisons::LibraryContracts {
            report: self.report,
            facts: &mut self.facts,
        }
        .clone_contract(declaration)?;
        for id in [
            declaration.definition,
            declaration.self_parameter,
            declaration.method,
        ] {
            self.states[id.0 as usize] = State::Complete;
        }
        Ok(())
    }
}
impl super::comparisons::LibraryContracts<'_> {
    fn clone_contract(&mut self, declaration: BuiltinCloning) -> Result<(), Error> {
        let source = declaration.origin;
        if self
            .report
            .resolution
            .builtin_traits
            .get(crate::resolve::model::BuiltinTrait::Clone)
            != Some(declaration.definition)
            || self.facts.cloning.is_some()
        {
            return Err(self.error(source, "Clone declaration differs from canonical demand"));
        }
        let (subject, application) = self.library_trait_owner(
            source,
            declaration.definition,
            declaration.self_parameter,
            true,
        )?;
        let own = Predicate {
            ty: subject,
            bound: Bound::Trait(application),
        };
        let builtins = [Bound::Clone, Bound::Sized].map(|bound| Written {
            source,
            value: Predicate { ty: subject, bound }.into(),
        });
        self.facts.generics.predicates.insert(
            declaration.definition,
            std::iter::once(Written {
                source,
                value: own.into(),
            })
            .chain(builtins.iter().copied())
            .collect(),
        );
        self.facts
            .generics
            .traits
            .get_or_insert_default()
            .declare(
                ItemId(declaration.definition.0),
                Header {
                    self_type: subject,
                    self_bounds: std::collections::HashSet::from([source]),
                    parameters: Vec::new(),
                    predicates: builtins.to_vec(),
                    results: Vec::new(),
                    regions: Vec::new(),
                },
            )
            .map_err(|e| self.error(source, e))?;
        let signature = self.clone_signature(source, subject, 0, 1)?;
        self.register_library_method(
            super::comparison_methods::MethodContract {
                origin: source,
                owner: declaration.definition,
                self_parameter: declaration.self_parameter,
                definition: declaration.method,
                signature,
                output: None,
            },
            subject,
            &[],
        )?;
        self.facts.cloning = Some(Box::new(crate::sema::cloning::Registry {
            canonical: declaration,
            original: None,
            complete: false,
            requested: false,
        }));
        Ok(())
    }
    pub(super) fn clone_signature(
        &mut self,
        source: super::NodeKey,
        subject: TypeId,
        borrow: u16,
        total: u32,
    ) -> Result<TypeId, Error> {
        let receiver = self.intern(
            source,
            Type::Reference {
                referent: subject,
                mutable: false,
                region: RegionId::bound(0, borrow).map_err(|e| self.error(source, e))?,
            },
        )?;
        let parameters = self
            .facts
            .types
            .intern_list(&[receiver])
            .map_err(|e| self.error(source, e))?;
        self.intern(
            source,
            Type::Function {
                parameters,
                result: subject,
                unsafe_: false,
                convention: crate::types::callable::Convention::RUST,
                bound_regions: total,
            },
        )
    }
}
