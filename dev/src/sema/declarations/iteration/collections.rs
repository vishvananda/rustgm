//! Collection impls publish real associated types and checked callable schemas.
pub(in crate::sema::declarations) mod conformance;
mod mutable;
mod parameters;
mod signatures;
use super::{Checker, DefId, Error, ItemId, State, Type};
use crate::{
    resolve::model::{BuiltinCollectionDeclaration, BuiltinIteration},
    sema::{
        associated::Schema,
        iteration::collections::{Original, Registry},
        traits::implementations::Implementation,
    },
    support::iteration::Collection,
    types::{
        GenericArgs, RegionId, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
pub(in crate::sema) fn owns(report: &crate::resolve::expansion::Report, owner: DefId) -> bool {
    report
        .resolution
        .builtin_traits
        .collections()
        .is_some_and(|c| {
            c.declarations().any(|d| {
                d.definition == owner
                    || d.method == owner
                    || d.item == Some(owner)
                    || d.into_iter == Some(owner)
            })
        })
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn activate_collection(
        &mut self,
        owner: DefId,
    ) -> Result<(), Error> {
        prepare(self.report, &mut self.facts, owner, false)?;
        let canonical = self
            .report
            .resolution
            .builtin_traits
            .collections()
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing collection catalog"))?;
        let kind = canonical
            .declarations()
            .find(|d| {
                d.definition == owner
                    || d.method == owner
                    || d.item == Some(owner)
                    || d.into_iter == Some(owner)
            })
            .map(|d| d.kind)
            .ok_or_else(|| {
                Error::at(canonical.origin.file, 0, "missing exact collection demand")
            })?;
        for library in [
            crate::support::library_types::LibraryType::SliceIter,
            crate::support::library_types::LibraryType::SliceIterMut,
        ] {
            let Some(nominal) = self.report.resolution.library_types.get(library) else {
                continue;
            };
            let item = ItemId(nominal.definition.0);
            if !self.nominal_registered(item)
                && let Some(definition) = self.facts.aggregates.get(item)
            {
                let (shape, repr) = (definition.kind, definition.repr);
                self.register_library_nominal(item, nominal.origin, shape, repr, &[])?;
            }
            if self.facts.aggregates.get(item).is_some() {
                for id in
                    std::iter::once(nominal.definition).chain(nominal.parameters.iter().copied())
                {
                    self.states[id.0 as usize] = State::Complete;
                }
            }
        }
        let nominal = self
            .report
            .resolution
            .library_types
            .get(if kind.mutable() {
                crate::support::library_types::LibraryType::SliceIterMut
            } else {
                crate::support::library_types::LibraryType::SliceIter
            })
            .ok_or_else(|| {
                Error::at(
                    canonical.origin.file,
                    0,
                    "missing iterator nominal identity",
                )
            })?;
        for id in std::iter::once(nominal.definition)
            .chain(nominal.parameters.iter().copied())
            .chain(
                canonical
                    .declarations()
                    .filter(|d| d.kind.mutable() == kind.mutable())
                    .flat_map(|d| {
                        [
                            Some(d.definition),
                            Some(d.parameter),
                            Some(d.method),
                            d.lifetime,
                            d.extent,
                            d.item,
                            d.into_iter,
                        ]
                        .into_iter()
                        .flatten()
                    }),
            )
        {
            self.states[id.0 as usize] = State::Complete;
        }
        Ok(())
    }
}
pub(in crate::sema::declarations) fn finish_prepared(
    report: &crate::resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
) -> Result<(), Error> {
    conformance::complete(report, facts, true)
}
pub(in crate::sema) fn prepare(
    report: &crate::resolve::expansion::Report,
    facts: &mut crate::sema::model::Declarations,
    owner: DefId,
    validated: bool,
) -> Result<(), Error> {
    if !owns(report, owner) {
        return Err(Error::at(
            super::super::FileId(0),
            0,
            "unregistered collection activation owner",
        ));
    }
    let canonical = report
        .resolution
        .builtin_traits
        .iteration()
        .ok_or_else(|| {
            Error::at(
                super::super::FileId(0),
                0,
                "missing canonical iteration contracts",
            )
        })?;
    if facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
        .is_none()
    {
        let mut builder = super::super::comparisons::LibraryContracts { report, facts };
        builder.collection_nominal(crate::support::library_types::LibraryType::SliceIter)?;
        builder.prepare_collection_contracts(canonical)?;
    }
    mutable::prepare(report, facts, owner)?;
    let late = facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|r| r.headers_validated());
    if validated && !late {
        return Err(Error::at(
            canonical.origin.file,
            0,
            "validated collection activation requires completed source implementations",
        ));
    }
    if validated || late {
        conformance::complete(report, facts, true)?;
    }
    Ok(())
}
impl super::super::comparisons::LibraryContracts<'_> {
    fn prepare_collection_contracts(&mut self, iteration: BuiltinIteration) -> Result<(), Error> {
        let canonical = self
            .report
            .resolution
            .builtin_traits
            .collections()
            .ok_or_else(|| self.error(iteration.origin, "missing collection catalog"))?;
        let canonical = crate::resolve::model::BuiltinCollections {
            origin: canonical.origin,
            iterator: canonical.iterator,
            declarations: canonical.declarations,
            mutable: None,
        };
        let mut originals = [None; 4];
        for declaration in canonical.declarations {
            originals[declaration.kind as usize] =
                Some(self.collection_contract(iteration, declaration)?);
        }
        let registry = Registry {
            status: crate::sema::iteration::collections::Status::Prepared,
            mutable: None,
            canonical,
            originals,
            prefixes: crate::support::hash::HashMap::default(),
        };
        self.facts
            .iteration
            .as_deref_mut()
            .ok_or_else(|| Error::at(iteration.origin.file, 0, "missing iteration registry"))?
            .collections = Some(Box::new(registry));
        Ok(())
    }
    fn collection_nominal(
        &mut self,
        kind: crate::support::library_types::LibraryType,
    ) -> Result<(), Error> {
        let nominal = self
            .report
            .resolution
            .library_types
            .get(kind)
            .ok_or_else(|| {
                Error::at(
                    super::super::FileId(0),
                    0,
                    "missing canonical iterator nominal",
                )
            })?;
        let item = ItemId(nominal.definition.0);
        if !self
            .facts
            .generics
            .nominal_contracts
            .contains(&nominal.definition)
        {
            self.library_nominal(kind)?;
        }
        let ty = self.facts.definitions[item.0 as usize]
            .ok_or_else(|| self.error(nominal.origin, "missing iterator self type"))?;
        let Some(Type::Adt(actual, arguments)) = self.facts.types.kind(ty) else {
            return Err(self.error(nominal.origin, "invalid iterator self type"));
        };
        if actual != item {
            return Err(self.error(nominal.origin, "iterator self type has a different owner"));
        }
        if self
            .facts
            .lifetimes
            .nominal_requirements
            .get(item)
            .is_none()
        {
            self.facts
                .lifetimes
                .nominal_requirements
                .append_leaf(&mut self.facts.types, &self.facts.aggregates, item)
                .map_err(|e| self.error(nominal.origin, e))?;
        }
        if self.facts.lifetimes.variances.get(item).is_none() {
            self.facts
                .lifetimes
                .variances
                .append_independent(&self.facts.types, &self.facts.aggregates, item, arguments)
                .map_err(|e| self.error(nominal.origin, e))?;
        }
        Ok(())
    }
    fn collection_contract(
        &mut self,
        iteration: BuiltinIteration,
        declaration: BuiltinCollectionDeclaration,
    ) -> Result<Original, Error> {
        let source = iteration.origin;
        let (parameter, subject, item, iterator) =
            self.collection_parameters(iteration, declaration)?;
        self.facts
            .associated
            .declare_language_trait(
                declaration.definition,
                Schema {
                    self_type: subject,
                    captures: u32::from(declaration.lifetime.is_some()),
                    bounds: Vec::new(),
                },
                None,
            )
            .map_err(|e| self.error(source, e))?;
        self.facts.definitions[declaration.definition.0 as usize] = Some(subject);
        for (member, ty) in declaration
            .item
            .into_iter()
            .map(|d| (d, item))
            .chain(declaration.into_iter.map(|d| (d, iterator)))
        {
            self.collection_type_member(iteration, declaration, member, ty)?;
        }
        let predicate = if let Some(trait_) = declaration.kind.trait_() {
            let application = self
                .facts
                .types
                .intern_trait_application(Application {
                    item: ItemId(iteration.get(trait_).definition.0),
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
                .ok_or_else(|| Error::at(source.file, 0, "collection header was not reserved"))?
                .prepare_factory(
                    &mut self.facts.types,
                    &self.facts.associated,
                    ItemId(iteration.get(trait_).definition.0),
                    Implementation {
                        owner: declaration.definition,
                        source,
                        predicate,
                        methods: Vec::new(),
                        types: Vec::new(),
                        constants: Vec::new(),
                    },
                )
                .map_err(|e| self.error(source, e))?;
            Some(predicate)
        } else {
            None
        };
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, declaration.definition)
            .map_err(|e| self.error(source, e))?;
        let signature = self.collection_method(iteration, declaration, subject, item, iterator)?;
        Ok(Original {
            declaration,
            parameter,
            subject,
            item,
            iterator,
            signature,
            predicate,
        })
    }
    fn collection_type_member(
        &mut self,
        iteration: BuiltinIteration,
        declaration: BuiltinCollectionDeclaration,
        member: DefId,
        ty: TypeId,
    ) -> Result<(), Error> {
        self.facts.definitions[member.0 as usize] = Some(ty);
        self.facts
            .generics
            .parents
            .insert(member, declaration.definition);
        self.facts.generics.parameters.insert(member, Vec::new());
        self.facts.associated.register_library_type_member(
            declaration.definition,
            member,
            self.report.resolution.definitions[member.0 as usize].name,
        );
        self.facts
            .associated
            .library_impl_member(declaration.definition, member)
            .map_err(|e| self.error(iteration.origin, e))?;
        Ok(())
    }
}
