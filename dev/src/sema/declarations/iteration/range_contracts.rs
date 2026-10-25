//! Integer range requests complete real canonical headers before body retry.
mod advance;
use super::{Checker, DefId, Error, ItemId, Type};
use crate::{
    resolve::{expansion::Report, model::BuiltinRangeImplementation},
    sema::{
        associated::Schema,
        iteration::range_originals::{Original, Registry},
        model::Declarations,
        traits::implementations::Implementation,
    },
    support::iteration::Trait,
    types::{
        GenericArgs,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
pub(in crate::sema) fn owns(report: &Report, owner: DefId) -> bool {
    report.resolution.builtin_traits.ranges().is_some_and(|r| {
        r.implementations
            .iter()
            .any(|d| [d.definition, d.item, d.method].contains(&owner))
    })
}
pub(in crate::sema) fn owns_header(report: &Report, owner: DefId) -> bool {
    report
        .resolution
        .builtin_traits
        .ranges()
        .is_some_and(|r| r.implementations.iter().any(|d| d.definition == owner))
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn activate_range(
        &mut self,
        owner: DefId,
    ) -> Result<(), Error> {
        prepare(self.report, &mut self.facts, owner, false)?;
        let canonical = self
            .report
            .resolution
            .builtin_traits
            .ranges()
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing range catalog"))?;
        for d in canonical.implementations {
            for id in [d.definition, d.item, d.method] {
                self.states[id.0 as usize] = super::super::State::Complete;
            }
        }
        Ok(())
    }
    pub(in crate::sema::declarations) fn validate_range_impl(
        &mut self,
        owner: DefId,
    ) -> Result<bool, Error> {
        if !self
            .facts
            .iteration
            .as_deref()
            .and_then(|r| r.ranges.as_deref())
            .is_some_and(|r| {
                r.originals
                    .iter()
                    .any(|o| o.declaration.definition == owner)
            })
        {
            return Ok(false);
        }
        complete(self.report, &mut self.facts, false)?;
        Ok(true)
    }
}
pub(in crate::sema) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    owner: DefId,
    validated: bool,
) -> Result<(), Error> {
    if !owns(report, owner) {
        return Err(Error::at(
            super::super::FileId(0),
            0,
            "unregistered range request",
        ));
    }
    let canonical = report
        .resolution
        .builtin_traits
        .ranges()
        .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing canonical ranges"))?;
    if facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .is_none()
    {
        let mut builder = super::super::comparisons::LibraryContracts { report, facts };
        let mut originals = Vec::with_capacity(canonical.implementations.len());
        for declaration in canonical.implementations {
            originals.push(builder.range_original(declaration)?);
        }
        builder
            .facts
            .iteration
            .as_deref_mut()
            .ok_or_else(|| Error::at(canonical.origin.file, 0, "missing iteration owner"))?
            .ranges = Some(Box::new(Registry {
            canonical: canonical.clone(),
            originals,
            complete: false,
        }));
    }
    let late = facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|r| r.headers_validated());
    if validated && !late {
        return Err(Declarations::error(
            report,
            canonical.origin,
            "validated range request precedes ordinary headers",
        ));
    }
    if validated || late {
        complete(report, facts, true)?;
    }
    Ok(())
}
pub(in crate::sema::declarations) fn complete(
    report: &Report,
    facts: &mut Declarations,
    overlap: bool,
) -> Result<(), Error> {
    let Some(registry) = facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .filter(|r| !r.complete)
    else {
        return Ok(());
    };
    let source = registry.canonical.origin;
    let originals = registry.originals.clone();
    let canonical = facts
        .iteration
        .as_deref()
        .ok_or_else(|| Error::at(source.file, 0, "missing iteration registry"))?
        .canonical
        .get(Trait::Iterator);
    if overlap {
        super::collections::conformance::check_overlap(report, facts, source)?;
    }
    let mut builder = super::super::comparisons::LibraryContracts { report, facts };
    for original in originals {
        if !builder
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|i| i.is_validated(original.declaration.definition))
        {
            builder.check_library_iteration_impl(
                source,
                original.declaration.definition,
                0,
                original.predicate,
                &[(canonical.item, original.declaration.item)],
                (canonical.method, original.declaration.method),
            )?;
        }
    }
    builder
        .facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.ranges.as_deref_mut())
        .ok_or_else(|| Error::at(source.file, 0, "missing range completion owner"))?
        .complete = true;
    Ok(())
}
impl super::super::comparisons::LibraryContracts<'_> {
    fn range_original(
        &mut self,
        declaration: BuiltinRangeImplementation,
    ) -> Result<Original, Error> {
        let canonical = self
            .report
            .resolution
            .builtin_traits
            .ranges()
            .ok_or_else(|| Error::at(super::super::FileId(0), 0, "missing range identities"))?;
        let source = canonical.origin;
        if !canonical.implementations.contains(&declaration)
            || !self
                .report
                .resolution
                .primitives
                .values()
                .any(|&id| id == declaration.primitive)
        {
            return Err(self.error(source, "invalid canonical range primitive/owner"));
        }
        let element = self.facts.definitions[declaration.primitive.0 as usize]
            .ok_or_else(|| self.error(source, "missing range primitive type"))?;
        if !matches!(self.facts.types.kind(element),Some(Type::Primitive(p))if p.integer().is_some())
        {
            return Err(self.error(source, "range primitive is not integer"));
        }
        let args = self
            .facts
            .types
            .intern_arguments(&[Argument::Type(element)])
            .map_err(|e| self.error(source, e))?;
        let subject = self.intern(source, Type::Adt(ItemId(canonical.nominal.0), args))?;
        let iteration = self
            .facts
            .iteration
            .as_deref()
            .ok_or_else(|| self.error(source, "missing Iterator declaration"))?
            .canonical;
        let expected = iteration.get(Trait::Iterator);
        let predicate = self.range_impl(source, declaration, subject, element, expected)?;
        let signature =
            self.range_method(source, declaration, subject, element, iteration.option)?;
        Ok(Original {
            declaration,
            element,
            subject,
            signature,
            predicate,
        })
    }
    fn range_impl(
        &mut self,
        source: crate::resolve::unit::NodeKey,
        declaration: BuiltinRangeImplementation,
        subject: crate::types::TypeId,
        element: crate::types::TypeId,
        expected: crate::resolve::model::BuiltinIterationDeclaration,
    ) -> Result<Predicate, Error> {
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
        self.facts.definitions[declaration.definition.0 as usize] = Some(subject);
        self.facts
            .generics
            .parameters
            .insert(declaration.definition, Vec::new());
        self.facts.definitions[declaration.item.0 as usize] = Some(element);
        self.facts
            .generics
            .parameters
            .insert(declaration.item, Vec::new());
        self.facts
            .generics
            .parents
            .insert(declaration.item, declaration.definition);
        self.facts
            .associated
            .register_library_type_member(declaration.definition, declaration.item);
        self.facts
            .associated
            .library_impl_member(declaration.definition, declaration.item)
            .map_err(|e| self.error(source, e))?;
        let application = self
            .facts
            .types
            .intern_trait_application(Application {
                item: ItemId(expected.definition.0),
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
            .ok_or_else(|| Error::at(source.file, 0, "range headers were not reserved"))?
            .prepare_factory(
                &mut self.facts.types,
                &self.facts.associated,
                ItemId(expected.definition.0),
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
        self.facts
            .generics
            .complete_environment(&mut self.facts.types, declaration.definition)
            .map_err(|e| self.error(source, e))?;
        Ok(predicate)
    }
}

pub(in crate::sema) fn prepare_dependency(
    report: &Report,
    facts: &mut Declarations,
    dependency: crate::sema::traits::selection::Dependency,
    availability: crate::sema::traits::implementations::Availability,
) -> Result<bool, &'static str> {
    let crate::sema::traits::selection::Dependency::Header(owner) = dependency else {
        return Ok(false);
    };
    if !owns_header(report, owner) {
        return Ok(false);
    }
    if facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|i| i.header(owner).is_some())
    {
        return Err("range readiness requested an already prepared header");
    }
    prepare(
        report,
        facts,
        owner,
        matches!(
            availability,
            crate::sema::traits::implementations::Availability::Validated
        ),
    )
    .map_err(|_| "canonical range iteration preparation failed")?;
    Ok(true)
}
