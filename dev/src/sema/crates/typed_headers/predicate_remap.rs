//! Canonical source inputs are derived once, never supplied by site-only claims.
use super::{Error, Headers, error};
use crate::{
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        crates::Imported,
        model::{DefId, DefKind},
    },
    sema::declaration_inputs::Input,
    support::crates::{SourceDescriptor, generics::source as s},
    types::{
        RegionId, Store, TypeId,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
    },
};
mod application_rows;
use std::sync::Arc;

/// The graph is the canonical remap owned by the same staged header ingest.
/// It is not a new declaration or body index and never carries paid proof.
pub(super) struct Context<'a> {
    imported: &'a Imported,
    catalog: &'a Catalog,
    descriptor: SourceDescriptor<'a>,
    graph: &'a super::mapped_source::Graph,
    applications: Vec<Option<Predicate>>,
}

/// Fields stay private so arbitrary predicates cannot acquire canonical-prefix
/// status by retaining a valid source site. Only the typed remapper constructs it.
pub(in crate::sema) struct Prepared {
    declaration: ImportedDeclarationId,
    inputs: Arc<[Input]>,
    self_subjects: Box<[bool]>,
}
impl Prepared {
    pub(super) fn all(&self) -> &Arc<[Input]> {
        &self.inputs
    }
    pub(in crate::sema) fn into_parts(self) -> (ImportedDeclarationId, Arc<[Input]>, Box<[bool]>) {
        (self.declaration, self.inputs, self.self_subjects)
    }
}
impl<'a> Context<'a> {
    pub(super) fn finish(
        mut self,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Vec<Predicate>, Error> {
        let count = self.applications.len();
        budget.reserve(count.saturating_mul(size_of::<Predicate>()))?;
        for index in 0..count {
            let index = u32::try_from(index)
                .map_err(|_| error("source trait application ordinal overflow"))?;
            self.application(s::TraitApplicationRef(index), store, budget)?;
        }
        let applications = self
            .applications
            .into_iter()
            .map(|entry| entry.ok_or_else(|| error("source trait application table is incomplete")))
            .collect::<Result<Vec<_>, _>>()?;
        budget.observed.vector(&applications);
        Ok(applications)
    }
    pub(super) fn new(
        imported: &'a Imported,
        catalog: &'a Catalog,
        descriptor: SourceDescriptor<'a>,
        graph: &'a super::mapped_source::Graph,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        budget.charge(1)?;
        let rows = descriptor.rows();
        graph.require(imported, catalog, descriptor, budget)?;
        let count = rows.trait_applications.len();
        budget.reserve(count.saturating_mul(size_of::<Option<Predicate>>()))?;
        Ok(Self {
            imported,
            catalog,
            descriptor,
            graph,
            applications: vec![None; count],
        })
    }
    pub(super) fn trait_inputs(
        &mut self,
        definition: DefId,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Prepared, Error> {
        budget.charge(1)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(declaration, definition, DefKind::Trait)
            .map_err(error)?;
        let (owner, ordinal, _) = self
            .catalog
            .declaration_coordinate(declaration)
            .map_err(error)?;
        if owner != self.descriptor.identity()
            || self
                .imported
                .generic_definition(owner, ordinal)
                .map_err(error)?
                != definition
        {
            return Err(error(
                "source trait input remap has a foreign catalog/descriptor owner",
            ));
        }
        let rows = self.descriptor.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(ordinal).map_err(error)? else {
            return Err(error(
                "source trait input remap lacks its actual trait header",
            ));
        };
        let header = &rows.traits[index as usize];
        let raw = header.inputs.slice(&rows.clauses).map_err(error)?;
        let canonical = header.canonical_inputs as usize;
        if canonical > raw.len() {
            return Err(error("source trait inputs truncate their canonical prefix"));
        }
        budget.reserve(
            raw.len()
                .saturating_mul(2 * size_of::<Input>())
                .saturating_add(canonical.saturating_mul(2 * size_of::<bool>()))
                .saturating_add(2 * size_of::<usize>()),
        )?;
        let mut inputs = Vec::with_capacity(raw.len());
        let mut self_subjects = Vec::with_capacity(canonical);
        for (index, raw) in raw.iter().enumerate() {
            budget.charge(1)?;
            if raw.owner != ordinal || (index >= canonical && raw.self_subject.is_some()) {
                return Err(error(
                    "source trait input changed canonical owner/prefix membership",
                ));
            }
            let value = self.clause(raw, store, budget)?;
            let site = self
                .catalog
                .declaration_site(declaration, raw.site)
                .map_err(error)?;
            inputs.push(Input {
                source: self
                    .catalog
                    .declaration_proof_site(declaration, site)
                    .map_err(error)?,
                value,
            });
            if index < canonical {
                self_subjects.push(raw.self_subject.is_some());
            }
        }
        budget.observed.add(
            raw.len()
                .saturating_mul(size_of::<Input>())
                .saturating_add(2 * size_of::<usize>())
                .saturating_add(self_subjects.len().saturating_mul(size_of::<bool>())),
        );
        Ok(Prepared {
            declaration,
            inputs: Arc::from(inputs),
            self_subjects: self_subjects.into_boxed_slice(),
        })
    }
    pub(super) fn declaration_inputs(
        &mut self,
        definition: DefId,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Arc<[Input]>, Error> {
        budget.charge(1)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        let (owner, ordinal, _) = self
            .catalog
            .declaration_coordinate(declaration)
            .map_err(error)?;
        if owner != self.descriptor.identity()
            || self
                .imported
                .generic_definition(owner, ordinal)
                .map_err(error)?
                != definition
        {
            return Err(error(
                "source function inputs belong to another actual import owner",
            ));
        }
        let rows = self.descriptor.rows();
        let (kind, inputs) = match rows.header(ordinal).map_err(error)? {
            Some(s::HeaderRef::Function(index)) => {
                (DefKind::Function, rows.functions[index as usize].inputs)
            }
            Some(s::HeaderRef::Implementation(index)) => (
                DefKind::SelfType,
                rows.implementations[index as usize].inputs,
            ),
            _ => {
                return Err(error(
                    "source declaration inputs lack their exact completed header",
                ));
            }
        };
        self.catalog
            .validate_declaration(declaration, definition, kind)
            .map_err(error)?;
        let raw = inputs.slice(&rows.clauses).map_err(error)?;
        budget.reserve(
            raw.len()
                .saturating_mul(2 * size_of::<Input>())
                .saturating_add(2 * size_of::<usize>()),
        )?;
        let mut inputs = Vec::with_capacity(raw.len());
        for clause in raw {
            budget.charge(1)?;
            if clause.owner != ordinal || clause.self_subject.is_some() {
                return Err(error(
                    "source declaration input invents a foreign/canonical Self premise",
                ));
            }
            let value = self.clause(clause, store, budget)?;
            let site = self
                .catalog
                .declaration_site(declaration, clause.site)
                .map_err(error)?;
            inputs.push(Input {
                source: self
                    .catalog
                    .declaration_proof_site(declaration, site)
                    .map_err(error)?,
                value,
            });
        }
        budget.observed.add(
            inputs
                .len()
                .saturating_mul(size_of::<Input>())
                .saturating_add(2 * size_of::<usize>()),
        );
        Ok(Arc::from(inputs))
    }
    fn clause(
        &mut self,
        raw: &s::Clause,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Clause, Error> {
        let predicate = match raw.requirement {
            s::Requirement::Trait {
                application,
                effect,
            } => {
                let mut predicate = self.application(application, store, budget)?;
                let Bound::Trait(application) = predicate.bound else {
                    return Err(error(
                        "source clause remap changed its ordinary application cache",
                    ));
                };
                predicate.bound = Bound::trait_application(application, effect);
                predicate
            }
            s::Requirement::TypeOutlives { subject, shorter } => Predicate {
                ty: self.ty(subject)?,
                bound: Bound::Outlives(self.region(shorter)?),
            },
            s::Requirement::Outlives(_) => {
                return Err(error(
                    "regional clauses require their separate header relation remap",
                ));
            }
        };
        if let Some(binder) = raw.binder {
            let rows = self.descriptor.rows();
            let binder = rows
                .binders
                .get(binder.0 as usize)
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
    fn ty(&self, ty: crate::support::crates::generics::TypeRef) -> Result<TypeId, Error> {
        self.graph.ty(ty)
    }
    fn region(&self, region: s::RegionRef) -> Result<RegionId, Error> {
        self.graph.region(region)
    }
}
