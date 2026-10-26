//! Canonical imported headers retain actual catalog ownership and declared inputs.
mod applications;
mod consumer_sessions;
mod ingest;
pub(super) mod mapped_source;
pub(in crate::sema) mod predicate_remap;
#[path = "typed_headers/demanded_types.rs"]
mod recipe_cursor;
mod roots;
#[path = "typed_headers/demanded_signatures.rs"]
mod signature_shapes;
mod staged_headers;
mod staged_ingest;
#[path = "typed_headers/dormant_session.rs"]
mod unpaid_provided;
use super::{Declarations, Error};
use crate::{
    resolve::{
        body_origin::{BodyOrigin, DeclarationOrigin},
        crates::Imported,
        model::DefId,
    },
    sema::{
        declaration_inputs::Input,
        function_headers::{DeclarationHeader, FunctionHeader},
    },
    support::{crates::CrateIdentity, hash::HashMap},
    types::{TypeId, parameters::environment::Environment},
};
use std::sync::Arc;
fn error(message: &'static str) -> Error {
    Error::unlocated(message)
}
#[cfg(test)]
#[path = "../../../tests/invariants/origin_relationships.rs"]
mod origin_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/header_reservations.rs"]
mod tests;

pub(super) struct Header {
    pub declaration: DeclarationOrigin,
    pub body: Option<BodyOrigin>,
    pub signature: TypeId,
    pub callable: TypeId,
    pub environment: Arc<Environment>,
    pub inputs: Arc<[Input]>,
    pub declared: usize,
    pub universals: u32,
    pub constant: bool,
}
#[derive(Default)]
pub(in crate::sema) struct Headers {
    entries: HashMap<DefId, Header>,
    graphs: HashMap<CrateIdentity, Vec<TypeId>>,
    applications: HashMap<CrateIdentity, Vec<applications::Application>>,
    source: Option<std::rc::Rc<mapped_source::Graph>>,
    bytes: usize,
    work: usize,
    base: usize,
    observed: crate::support::retained::Allocation,
}
impl Headers {
    pub(in crate::sema) fn reserve(&mut self, growth: usize) -> Result<(), Error> {
        self.bytes = self.bytes.saturating_add(growth);
        if self.bytes.saturating_add(self.base) > crate::support::crates::MAX_BYTES {
            return Err(error(
                "canonical generic headers exceed combined retained storage budget",
            ));
        }
        Ok(())
    }
    fn prepare(&mut self, rows: &crate::support::crates::generics::Rows) -> Result<(), Error> {
        let growth = rows
            .types
            .len()
            .saturating_mul(size_of::<TypeId>())
            .saturating_add(
                rows.functions
                    .len()
                    .saturating_mul(4 * size_of::<(DefId, Header)>()),
            )
            .saturating_add(rows.clauses.len().saturating_mul(size_of::<Input>()))
            .saturating_add(
                rows.declarations
                    .len()
                    .saturating_mul(size_of::<Option<TypeId>>() + size_of::<roots::Graph>()),
            )
            .saturating_add(rows.roots.len().saturating_mul(size_of::<TypeId>()))
            .saturating_add(crate::support::retained::new_map::<
                CrateIdentity,
                Vec<TypeId>,
            >(1));
        self.reserve(growth)
    }
    pub(in crate::sema) fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.work = self.work.saturating_add(count);
        if self.work > crate::support::crates::MAX_COMPONENTS {
            return Err(error("imported generic header work budget exceeded"));
        }
        Ok(())
    }
    fn symbol(&mut self, identity: &crate::support::crates::ExternalDef) -> Result<String, Error> {
        let mut length = 4_usize;
        for name in std::iter::once(identity.owner().name())
            .chain(std::iter::once(identity.owner().disambiguator()))
            .chain(identity.path().iter().map(String::as_str))
        {
            self.charge(name.len().saturating_add(1))?;
            length = length.saturating_add(2 * name.len() + 22);
        }
        let growth = 4 * length.saturating_add(size_of::<(DefId, String)>());
        self.reserve(growth)?;
        Ok(identity.rust_symbol())
    }
    pub(super) fn get(&self, definition: DefId) -> Option<&Header> {
        self.entries.get(&definition)
    }
    fn types(&self, owner: &CrateIdentity) -> Result<&[TypeId], &'static str> {
        self.graphs
            .get(owner)
            .map(Vec::as_slice)
            .ok_or("missing imported original type graph")
    }
}
impl super::Linkage {
    pub(in crate::sema) fn require_original_store(
        &self,
        store: &crate::types::Store,
    ) -> Result<(), &'static str> {
        if let Some(graph) = self
            .originals
            .as_deref()
            .and_then(|headers| headers.source.as_deref())
        {
            graph.require_store(store)?;
        }
        Ok(())
    }
    pub(in crate::sema) fn require_source_declaration(
        &self,
        facts: &Declarations,
        definition: DefId,
        scheme: &crate::sema::lifetimes::FunctionRegions,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(1);
        self.original_declaration_header(definition)?
            .ok_or("source method lost its actual completed declaration header")?;
        let headers = self
            .originals
            .as_deref()
            .ok_or("source method lacks its authenticated canonical headers")?;
        let graph = headers
            .source
            .as_deref()
            .ok_or("source method lacks its authenticated canonical owner graph")?;
        let catalog = self
            .origin_catalog()
            .ok_or("source method lacks its actual catalog")?;
        graph.require_method_regions(catalog, definition, scheme, steps)?;
        // Ordinary Graph headers retain their established eager relationship.
        // A private demanded declaration must use its own checked scope owner.
        let header = headers
            .get(definition)
            .ok_or("source method lost its actual completed original header")?;
        graph.require_method_environment(catalog, facts, definition, header, steps)
    }

    pub(crate) fn original_work(&self) -> Option<(usize, usize, usize, usize)> {
        self.originals
            .as_ref()
            .map(|h| (h.work, h.bytes, h.observed.bytes, h.observed.visits))
    }
    pub(crate) fn original_callable(&self, definition: DefId) -> Option<(TypeId, bool)> {
        self.originals
            .as_deref()?
            .get(definition)
            .map(|h| (h.callable, h.constant))
    }
    pub(crate) fn original_types(&self, owner: &CrateIdentity) -> Result<&[TypeId], &'static str> {
        self.originals
            .as_deref()
            .ok_or("imported original headers are absent")?
            .types(owner)
    }
    fn checked_original_header(&self, definition: DefId) -> Result<Option<&Header>, &'static str> {
        let Some(header) = self.originals.as_deref().and_then(|h| h.get(definition)) else {
            return Ok(None);
        };
        let catalog = self
            .origin_catalog()
            .ok_or("imported function has no ownership catalog")?;
        let DeclarationOrigin::Imported(declaration) = header.declaration else {
            return Err("imported header changed its declaration provenance");
        };
        catalog.validate_declaration(
            declaration,
            definition,
            crate::resolve::model::DefKind::Function,
        )?;
        let source = self
            .originals
            .as_deref()
            .and_then(|h| h.source.as_deref())
            .map(|source| source.function(catalog, definition, header.body, header.universals))
            .transpose()?
            .unwrap_or(false);
        if !source {
            if let Some(body) = header.body {
                catalog.validate_function_origin(definition, body)?;
            } else {
                return Err("imported required method lacks its authenticated source header");
            }
        }
        Ok(Some(header))
    }
    pub(crate) fn original_declaration_header(
        &self,
        definition: DefId,
    ) -> Result<Option<DeclarationHeader>, &'static str> {
        Ok(self
            .checked_original_header(definition)?
            .map(|header| DeclarationHeader {
                declaration: header.declaration,
                body: header.body,
                signature: header.signature,
                universals: header.universals,
            }))
    }
    pub(crate) fn original_header(
        &self,
        definition: DefId,
    ) -> Result<Option<FunctionHeader<'_>>, &'static str> {
        let Some(header) = self.checked_original_header(definition)? else {
            return Ok(None);
        };
        Ok(Some(FunctionHeader {
            declaration: header.declaration,
            body: header.body,
            signature: header.signature,
            universals: header.universals,
            environment: Some(&header.environment),
        }))
    }
    pub(crate) fn original_application(
        &self,
        owner: &CrateIdentity,
        application: crate::support::crates::generics::ApplicationRef,
    ) -> Result<&applications::Application, &'static str> {
        let row = self
            .originals
            .as_deref()
            .and_then(|h| h.applications.get(owner))
            .and_then(|v| v.get(application.0 as usize))
            .ok_or("missing canonical imported application")?;
        self.original_declaration_header(row.target)?
            .ok_or("application target header disappeared")?;
        Ok(row)
    }
    pub(crate) fn original_inputs(
        &self,
        definition: DefId,
        declared: bool,
    ) -> Result<&[Input], &'static str> {
        let Some(header) = self.originals.as_deref().and_then(|h| h.get(definition)) else {
            let Some(source) = self.originals.as_deref().and_then(|h| h.source.as_deref()) else {
                return Ok(&[]);
            };
            let catalog = self
                .origin_catalog()
                .ok_or("source declaration inputs lack their actual catalog")?;
            return Ok(source
                .declaration_inputs(catalog, definition, declared)?
                .unwrap_or(&[]));
        };
        self.original_declaration_header(definition)?
            .ok_or("imported input header disappeared")?;
        Ok(if declared {
            &header.inputs[..header.declared]
        } else {
            &header.inputs
        })
    }
}
pub(crate) fn is_source_original(
    facts: &Declarations,
    definition: DefId,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    *steps = steps.saturating_add(1);
    let Some(linkage) = facts.crates.as_deref() else {
        return Ok(false);
    };
    let Some(graph) = linkage
        .originals
        .as_deref()
        .and_then(|headers| headers.source.as_deref())
    else {
        return Ok(false);
    };
    graph.require_store(&facts.types)?;
    let catalog = linkage
        .origin_catalog()
        .ok_or("source classification lacks its actual catalog")?;
    graph.owns_original(catalog, definition, steps)
}

pub(crate) fn source_original<'a>(
    facts: &'a Declarations,
    definition: DefId,
    steps: &mut usize,
) -> Result<Option<mapped_source::SourceOriginal<'a>>, &'static str> {
    let Some(linkage) = facts.crates.as_deref() else {
        return Ok(None);
    };
    let Some(headers) = linkage.originals.as_deref() else {
        return Ok(None);
    };
    let Some(graph) = headers.source.as_deref() else {
        return Ok(None);
    };
    *steps = steps.saturating_add(1);
    let catalog = linkage
        .origin_catalog()
        .ok_or("source original lacks its actual current catalog")?;
    graph.source_original(catalog, facts, headers, definition, steps)
}

pub(in crate::sema) fn source_trait_header<'a>(
    facts: &'a Declarations,
    role: crate::sema::compiler_items::Role,
    steps: &mut usize,
) -> Result<crate::sema::compiler_items::headers::Trait<'a>, &'static str> {
    *steps = steps.saturating_add(1);
    let linkage = facts
        .crates
        .as_deref()
        .ok_or("imported trait lacks actual linkage")?;
    let graph = linkage
        .originals
        .as_deref()
        .and_then(|h| h.source.as_deref())
        .ok_or("imported trait requires its actual canonical source graph")?;
    let catalog = linkage
        .origin_catalog()
        .ok_or("imported trait lacks its actual catalog")?;
    graph.trait_header(catalog, facts, role, steps)
}

pub(in crate::sema) fn charge_source_headers(
    facts: &mut Declarations,
    steps: usize,
    bytes: usize,
) -> Result<(), Error> {
    let headers = facts
        .crates
        .as_deref_mut()
        .and_then(|l| l.originals.as_deref_mut())
        .ok_or_else(|| error("source validation lacks its installed header budget"))?;
    if headers.source.is_none() {
        return Err(error(
            "source validation cannot use legacy header authority",
        ));
    }
    headers.charge(steps)?;
    headers.reserve(bytes)?;
    headers.observed.add(bytes);
    Ok(())
}

/// Charge attempted classification at its mutable caller, including rejection.
pub(in crate::sema) fn source_readiness(
    facts: &mut Declarations,
    definition: DefId,
) -> Result<Option<mapped_source::Readiness>, Error> {
    let mut steps = 0;
    let result = (|| {
        let Some(linkage) = facts.crates.as_deref() else {
            return Ok(None);
        };
        let Some(headers) = linkage.originals.as_deref() else {
            return Ok(None);
        };
        let Some(graph) = headers.source.as_deref() else {
            return Ok(None);
        };
        steps += 1;
        let catalog = linkage
            .origin_catalog()
            .ok_or("source readiness lacks its actual semantic catalog")?;
        graph
            .readiness(catalog, facts, headers, definition, &mut steps)
            .map(Some)
    })();
    if let Some(headers) = facts
        .crates
        .as_deref_mut()
        .and_then(|l| l.originals.as_deref_mut())
    {
        headers.charge(steps)?;
    }
    result.map_err(error)
}

pub(super) fn import(
    report: &crate::resolve::expansion::Report,
    imported: &Imported,
    facts: &mut Declarations,
) -> Result<(), Error> {
    if !imported
        .dependencies()
        .artifacts()
        .iter()
        .any(|a| a.generic_payload().is_some())
    {
        return Ok(());
    }
    if imported.dependencies().artifacts().iter().any(|artifact| {
        artifact.generic_payload().is_some_and(|payload| {
            matches!(
                payload.view(),
                crate::support::crates::generics::PayloadView::Source(_)
            )
        })
    }) {
        return staged_ingest::import(report, imported, facts);
    }
    import_legacy(imported, facts)
}

fn import_legacy(imported: &Imported, facts: &mut Declarations) -> Result<(), Error> {
    if facts.uses_source_copy() {
        return Err(error(
            "generic row version zero cannot import into a source language-role provider",
        ));
    }
    let catalog = imported
        .origin_catalog()
        .map_err(error)?
        .ok_or_else(|| error("generic original import lacks its completed resolver catalog"))?;
    let mut headers = Headers {
        base: imported.dependencies().artifacts().iter().fold(
            catalog.retained_bytes(),
            |bytes, artifact| {
                bytes.saturating_add(
                    artifact
                        .generic_payload()
                        .map_or(0, crate::support::crates::generics::Payload::retained_bytes),
                )
            },
        ),
        ..Headers::default()
    };
    headers.reserve(size_of::<Headers>())?;
    for artifact in imported.dependencies().artifacts() {
        let Some(payload) = artifact.generic_payload() else {
            continue;
        };
        let rows = payload.legacy_rows().map_err(error)?;
        headers.prepare(rows)?;
        let mut types = roots::import(imported, facts, artifact.identity(), rows, &mut headers)?;
        for function in &rows.functions {
            headers.charge(1)?;
            let context = ingest::Context {
                imported,
                catalog: &catalog,
                owner: artifact.identity(),
                rows,
                types: &types,
            };
            let (definition, header) = ingest::function(&context, facts, function, &mut headers)?;
            if headers.entries.insert(definition, header).is_some() {
                return Err(error("duplicate canonical imported header"));
            }
        }
        if rows.calls.is_some() {
            headers.reserve(crate::support::retained::new_map::<
                CrateIdentity,
                Vec<applications::Application>,
            >(1))?;
            let calls = applications::import(
                imported,
                facts,
                artifact.identity(),
                payload,
                &mut types,
                &mut headers,
            )?;
            headers
                .applications
                .insert(artifact.identity().clone(), calls);
        }
        roots::locals(
            imported,
            facts,
            artifact.identity(),
            rows,
            &types,
            &mut headers,
        )?;
        headers
            .observed
            .add(crate::support::retained::vector(&types.nodes));
        headers
            .graphs
            .insert(artifact.identity().clone(), types.nodes);
    }
    let linkage = facts.crates.get_or_insert_default();
    headers.observed.map(&headers.entries);
    headers.observed.map(&headers.graphs);
    headers.observed.map(&headers.applications);
    headers.observed.add(size_of::<Headers>());
    if headers.observed.bytes > headers.bytes {
        return Err(error(
            "imported header payload exceeded its allocation preflight",
        ));
    }
    if linkage.originals.replace(Box::new(headers)).is_some() {
        return Err(error("imported original headers were already published"));
    }
    Ok(())
}
