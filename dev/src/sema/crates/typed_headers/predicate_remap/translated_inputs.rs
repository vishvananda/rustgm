//! One canonical translator; its sealed ports retain actual defining ownership.
use super::{Error, Headers, Input, RegionId, Store, TypeId, error, s};
use crate::{
    resolve::{
        body_origin::{ImportedDeclarationId, ProofSite},
        model::{DefId, DefKind},
    },
    sema::crates::typed_headers::recipe_cursor::Budget,
    support::crates::{generics as g, owner_catalog::DeclarationRef},
    types::{
        GenericArgs, ItemId,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};
use std::sync::Arc;

pub(in crate::sema::crates::typed_headers) mod sealed {
    pub trait Sealed {}
}
pub(in crate::sema::crates::typed_headers) trait Mapping<'rows, B: Budget>:
    sealed::Sealed
{
    fn rows(&self) -> s::ValidatedView<'rows>;
    fn trait_definition(&self, row: DeclarationRef, budget: &mut B) -> Result<DefId, Error>;
    fn site(&self, owner: DeclarationRef, site: u32, budget: &mut B) -> Result<ProofSite, Error>;
    fn ty(&mut self, store: &mut Store, ty: g::TypeRef, budget: &mut B) -> Result<TypeId, Error>;
    fn region(&mut self, region: s::RegionRef, budget: &mut B) -> Result<RegionId, Error>;
    fn arguments(
        &self,
        store: &mut Store,
        args: &[Argument],
        budget: &mut B,
    ) -> Result<GenericArgs, Error>;
    fn application(
        &self,
        store: &mut Store,
        application: Application,
        budget: &mut B,
    ) -> Result<crate::types::traits::ApplicationId, Error>;
}

pub(super) struct Ordinary<'a> {
    pub imported: &'a crate::resolve::crates::Imported,
    pub catalog: &'a crate::resolve::body_origin::Catalog,
    pub descriptor: crate::support::crates::SourceDescriptor<'a>,
    pub graph: &'a super::super::mapped_source::Graph,
    pub site_owner: Option<(DeclarationRef, ImportedDeclarationId)>,
}
impl sealed::Sealed for Ordinary<'_> {}
impl<'rows> Mapping<'rows, Headers> for Ordinary<'rows> {
    fn rows(&self) -> s::ValidatedView<'rows> {
        self.descriptor.rows()
    }
    fn trait_definition(&self, row: DeclarationRef, _: &mut Headers) -> Result<DefId, Error> {
        let definition = self
            .imported
            .generic_definition(self.descriptor.identity(), row)
            .map_err(error)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(declaration, definition, DefKind::Trait)
            .map_err(error)?;
        Ok(definition)
    }
    fn site(&self, owner: DeclarationRef, index: u32, _: &mut Headers) -> Result<ProofSite, Error> {
        let (actual, declaration) = self
            .site_owner
            .ok_or_else(|| error("source input translator has no authenticated site owner"))?;
        if actual != owner {
            return Err(error(
                "source input translator changed its actual site owner",
            ));
        }
        let site = self
            .catalog
            .declaration_site(declaration, index)
            .map_err(error)?;
        self.catalog
            .declaration_proof_site(declaration, site)
            .map_err(error)
    }
    fn ty(&mut self, _: &mut Store, ty: g::TypeRef, _: &mut Headers) -> Result<TypeId, Error> {
        self.graph.ty(ty)
    }
    fn region(&mut self, region: s::RegionRef, _: &mut Headers) -> Result<RegionId, Error> {
        self.graph.region(region)
    }
    fn arguments(
        &self,
        store: &mut Store,
        args: &[Argument],
        _: &mut Headers,
    ) -> Result<GenericArgs, Error> {
        store.intern_arguments(args).map_err(error)
    }
    fn application(
        &self,
        store: &mut Store,
        application: Application,
        _: &mut Headers,
    ) -> Result<crate::types::traits::ApplicationId, Error> {
        store.intern_trait_application(application).map_err(error)
    }
}

pub(in crate::sema::crates::typed_headers) fn application<'rows, B: Budget>(
    port: &mut impl Mapping<'rows, B>,
    cache: &mut [Option<Predicate>],
    reference: s::TraitApplicationRef,
    store: &mut Store,
    budget: &mut B,
) -> Result<Predicate, Error> {
    budget.charge(1)?;
    let index = reference.0 as usize;
    let cached = cache
        .get(index)
        .ok_or_else(|| error("source clause has an unknown trait application"))?;
    if let Some(predicate) = cached {
        return Ok(*predicate);
    }
    let rows = port.rows();
    let application = &rows.trait_applications[index];
    if application.declaration.artifact != rows.policy.provider {
        return Err(error(
            "source clause trait belongs to an unsupported dependency",
        ));
    }
    let definition = port.trait_definition(application.declaration.declaration, budget)?;
    let raw = application
        .arguments
        .slice(&rows.arguments)
        .map_err(error)?;
    budget.charge(raw.len().saturating_add(1))?;
    budget.reserve(raw.len().saturating_mul(4 * size_of::<Argument>()))?;
    let mut arguments = Vec::with_capacity(raw.len());
    for argument in raw {
        arguments.push(match *argument {
            s::Argument::Type(ty) => Argument::Type(port.ty(store, ty, budget)?),
            s::Argument::Region(region) => Argument::Lifetime(port.region(region, budget)?),
        });
    }
    let arguments = port.arguments(store, &arguments, budget)?;
    let id = port.application(
        store,
        Application {
            item: ItemId(definition.0),
            arguments,
        },
        budget,
    )?;
    let predicate = Predicate {
        ty: port.ty(store, application.subject, budget)?,
        bound: Bound::Trait(id),
    };
    cache[index] = Some(predicate);
    Ok(predicate)
}

fn clause<'rows, B: Budget>(
    port: &mut impl Mapping<'rows, B>,
    cache: &mut [Option<Predicate>],
    raw: &s::Clause,
    store: &mut Store,
    budget: &mut B,
) -> Result<Clause, Error> {
    let predicate = match raw.requirement {
        s::Requirement::Trait {
            application: reference,
            effect,
        } => {
            let mut predicate = application(port, cache, reference, store, budget)?;
            let Bound::Trait(application) = predicate.bound else {
                return Err(error(
                    "source clause remap changed its ordinary application cache",
                ));
            };
            predicate.bound = Bound::trait_application(application, effect);
            predicate
        }
        s::Requirement::TypeOutlives { subject, shorter } => Predicate {
            ty: port.ty(store, subject, budget)?,
            bound: Bound::Outlives(port.region(shorter, budget)?),
        },
        s::Requirement::Outlives(_) => {
            return Err(error(
                "regional clauses require their separate header relation remap",
            ));
        }
    };
    if let Some(reference) = raw.binder {
        let rows = port.rows();
        let binder = rows
            .binders
            .get(reference.0 as usize)
            .ok_or_else(|| error("source clause has an unknown actual binder"))?;
        if binder.owner != raw.owner || binder.kind != s::BinderKind::Clause {
            return Err(error("source clause binder changed its actual owner/kind"));
        }
        Ok(Clause::ForAll(
            store
                .intern_quantified(Quantified {
                    bound_regions: binder.count,
                    predicate,
                })
                .map_err(error)?,
        ))
    } else {
        Ok(Clause::Predicate(predicate))
    }
}

pub(in crate::sema::crates::typed_headers) fn inputs<'rows, B: Budget>(
    port: &mut impl Mapping<'rows, B>,
    cache: &mut [Option<Predicate>],
    owner: DeclarationRef,
    range: g::Range,
    canonical: Option<usize>,
    store: &mut Store,
    budget: &mut B,
) -> Result<(Arc<[Input]>, Box<[bool]>), Error> {
    let rows = port.rows();
    let raw = range.slice(&rows.clauses).map_err(error)?;
    let prefix = canonical.unwrap_or(0);
    if prefix > raw.len() {
        return Err(error("source trait inputs truncate their canonical prefix"));
    }
    budget.reserve(
        raw.len()
            .saturating_mul(2 * size_of::<Input>())
            .saturating_add(prefix.saturating_mul(2 * size_of::<bool>()))
            .saturating_add(2 * size_of::<usize>()),
    )?;
    let mut inputs = Vec::with_capacity(raw.len());
    let mut subjects = Vec::with_capacity(prefix);
    for (index, raw) in raw.iter().enumerate() {
        budget.charge(1)?;
        if raw.owner != owner || (index >= prefix && raw.self_subject.is_some()) {
            return Err(error(if canonical.is_some() {
                "source trait input changed canonical owner/prefix membership"
            } else {
                "source declaration input invents a foreign/canonical Self premise"
            }));
        }
        let value = clause(port, cache, raw, store, budget)?;
        inputs.push(Input {
            source: port.site(owner, raw.site, budget)?,
            value,
        });
        if index < prefix {
            subjects.push(raw.self_subject.is_some());
        }
    }
    Ok((Arc::from(inputs), subjects.into_boxed_slice()))
}
