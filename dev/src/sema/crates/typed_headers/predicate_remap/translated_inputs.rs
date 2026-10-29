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
#[path = "application_cells.rs"]
pub(in crate::sema::crates::typed_headers) mod applications;
use applications::Cache;
#[path = "unstable_names.rs"]
pub(in crate::sema::crates::typed_headers) mod feature_names;
pub(in crate::sema::crates::typed_headers) type InputRows = (Arc<[Input]>, Box<[bool]>);

pub(in crate::sema::crates::typed_headers) mod sealed {
    pub trait Sealed {}
}
pub(in crate::sema::crates::typed_headers) trait Mapping<'rows, B: Budget>:
    sealed::Sealed
{
    fn rows(&self) -> s::ValidatedView<'rows>;
    fn feature(
        &self,
        store: &mut Store,
        name: &str,
        budget: &mut B,
    ) -> Result<crate::types::parameters::features::FeatureId, Error>;
    fn trait_definition(&self, row: DeclarationRef, budget: &mut B) -> Result<DefId, Error>;
    fn trait_declaration(
        &self,
        row: DeclarationRef,
        budget: &mut B,
    ) -> Result<ImportedDeclarationId, Error>;
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
    fn feature(
        &self,
        store: &mut Store,
        name: &str,
        budget: &mut Headers,
    ) -> Result<crate::types::parameters::features::FeatureId, Error> {
        self.graph.require_store(store).map_err(error)?;
        if !self.descriptor.profile().feature_predicates() {
            return Err(error(
                "non-type feature import requires source metadata version four",
            ));
        }
        feature_names::intern(store, name, budget)
    }
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
    fn trait_declaration(
        &self,
        row: DeclarationRef,
        _: &mut Headers,
    ) -> Result<ImportedDeclarationId, Error> {
        // Context::trait_inputs authenticated this exact catalog/descriptor
        // correspondence before constructing the ordinary port.
        self.site_owner
            .filter(|(owner, _)| *owner == row)
            .map(|(_, declaration)| declaration)
            .ok_or_else(|| error("source trait inputs changed their authenticated declaration"))
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

pub(in crate::sema::crates::typed_headers) fn trait_inputs<'rows, B: Budget>(
    port: &mut impl Mapping<'rows, B>,
    cache: &mut impl Cache,
    owner: DeclarationRef,
    store: &mut Store,
    budget: &mut B,
) -> Result<super::Prepared, Error> {
    let rows = port.rows();
    let Some(s::HeaderRef::Trait(index)) = rows.header(owner).map_err(error)? else {
        return Err(error(
            "source trait input remap lacks its actual trait header",
        ));
    };
    let header = &rows.traits[index as usize];
    let declaration = port.trait_declaration(owner, budget)?;
    let (inputs, self_subjects) = inputs(
        port,
        cache,
        owner,
        header.inputs,
        Some(header.canonical_inputs as usize),
        store,
        budget,
    )?;
    Ok(super::Prepared {
        declaration,
        inputs,
        self_subjects,
    })
}

pub(in crate::sema::crates::typed_headers) fn application<'rows, B: Budget>(
    port: &mut impl Mapping<'rows, B>,
    cache: &mut impl Cache,
    reference: s::TraitApplicationRef,
    store: &mut Store,
    budget: &mut B,
) -> Result<Predicate, Error> {
    budget.charge(1)?;
    let index = reference.0 as usize;
    let cached = cache.entry(index)?;
    if let Some(predicate) = cached {
        return Ok(predicate);
    }
    let rows = port.rows();
    let application = &rows.trait_applications[index];
    if application.declaration.artifact != rows.policy.provider {
        return Err(error(
            "source clause trait belongs to an unsupported dependency",
        ));
    }
    let definition = port.trait_definition(application.declaration.declaration, budget)?;
    let record = application
        .arguments
        .slice(&rows.arguments)
        .map_err(error)?;
    budget.charge(record.len().saturating_add(1))?;
    budget.reserve(record.len().saturating_mul(4 * size_of::<Argument>()))?;
    let mut arguments = Vec::with_capacity(record.len());
    for argument in record {
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
    cache.publish(index, predicate)?;
    Ok(predicate)
}

fn clause<'rows, B: Budget>(
    port: &mut impl Mapping<'rows, B>,
    cache: &mut impl Cache,
    record: &s::Clause,
    store: &mut Store,
    budget: &mut B,
) -> Result<Clause, Error> {
    let predicate = match &record.requirement {
        s::Requirement::UnstableFeature(name) => {
            if record.binder.is_some() || record.self_subject.is_some() {
                return Err(error(
                    "feature requirement cannot carry a binder or Self subject",
                ));
            }
            return Ok(Clause::UnstableFeature(port.feature(store, name, budget)?));
        }
        s::Requirement::Trait {
            application: reference,
            effect,
        } => {
            let mut predicate = application(port, cache, *reference, store, budget)?;
            let Bound::Trait(application) = predicate.bound else {
                return Err(error(
                    "source clause remap changed its ordinary application cache",
                ));
            };
            predicate.bound = Bound::trait_application(application, *effect);
            predicate
        }
        s::Requirement::TypeOutlives { subject, shorter } => Predicate {
            ty: port.ty(store, *subject, budget)?,
            bound: Bound::Outlives(port.region(*shorter, budget)?),
        },
        s::Requirement::Outlives(_) => {
            return Err(error(
                "regional clauses require their separate header relation remap",
            ));
        }
    };
    if let Some(reference) = record.binder {
        let rows = port.rows();
        let binder = rows
            .binders
            .get(reference.0 as usize)
            .ok_or_else(|| error("source clause has an unknown actual binder"))?;
        if binder.owner != record.owner || binder.kind != s::BinderKind::Clause {
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
    cache: &mut impl Cache,
    owner: DeclarationRef,
    range: g::Range,
    canonical: Option<usize>,
    store: &mut Store,
    budget: &mut B,
) -> Result<InputRows, Error> {
    let rows = port.rows();
    let record = range.slice(&rows.clauses).map_err(error)?;
    let prefix = canonical.unwrap_or(0);
    if prefix > record.len() {
        return Err(error("source trait inputs truncate their canonical prefix"));
    }
    budget.reserve(
        record
            .len()
            .saturating_mul(2 * size_of::<Input>())
            .saturating_add(prefix.saturating_mul(2 * size_of::<bool>()))
            .saturating_add(2 * size_of::<usize>()),
    )?;
    let mut inputs = Vec::with_capacity(record.len());
    let mut subjects = Vec::with_capacity(prefix);
    for (index, record) in record.iter().enumerate() {
        budget.charge(1)?;
        if record.owner != owner || (index >= prefix && record.self_subject.is_some()) {
            return Err(error(if canonical.is_some() {
                "source trait input changed canonical owner/prefix membership"
            } else {
                "source declaration input invents a foreign/canonical Self premise"
            }));
        }
        let value = clause(port, cache, record, store, budget)?;
        inputs.push(Input {
            source: port.site(owner, record.site, budget)?,
            value,
        });
        if index < prefix {
            subjects.push(record.self_subject.is_some());
        }
    }
    Ok((Arc::from(inputs), subjects.into_boxed_slice()))
}
