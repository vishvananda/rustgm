//! Canonical imported headers retain actual catalog ownership and declared inputs.
mod ingest;
mod roots;
use super::{Declarations, Error};
use crate::{
    resolve::{
        body_origin::{BodyOrigin, DeclarationOrigin},
        crates::Imported,
        model::DefId,
    },
    sema::{declaration_inputs::Input, function_headers::FunctionHeader},
    support::{crates::CrateIdentity, hash::HashMap},
    types::{TypeId, parameters::environment::Environment},
};
use std::sync::Arc;
fn error(message: &'static str) -> Error {
    Error::unlocated(message)
}
#[cfg(test)]
#[path = "../../../tests/invariants/header_reservations.rs"]
mod tests;

pub(super) struct Header {
    pub declaration: DeclarationOrigin,
    pub body: BodyOrigin,
    pub signature: TypeId,
    pub callable: TypeId,
    pub environment: Arc<Environment>,
    pub inputs: Vec<Input>,
    pub declared: usize,
    pub constant: bool,
}
#[derive(Default)]
pub(super) struct Headers {
    entries: HashMap<DefId, Header>,
    graphs: HashMap<CrateIdentity, Vec<TypeId>>,
    bytes: usize,
    work: usize,
    base: usize,
    observed: crate::support::retained::Allocation,
}
impl Headers {
    fn reserve(&mut self, growth: usize) -> Result<(), Error> {
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
    fn charge(&mut self, count: usize) -> Result<(), Error> {
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
    pub(crate) fn original_header(
        &self,
        definition: DefId,
    ) -> Result<Option<FunctionHeader<'_>>, &'static str> {
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
        catalog.validate_function_origin(definition, header.body)?;
        Ok(Some(FunctionHeader {
            declaration: header.declaration,
            body: Some(header.body),
            signature: header.signature,
            universals: 0,
            environment: Some(&header.environment),
        }))
    }
    pub(crate) fn original_inputs(
        &self,
        definition: DefId,
        declared: bool,
    ) -> Result<&[Input], &'static str> {
        let Some(header) = self.originals.as_deref().and_then(|h| h.get(definition)) else {
            return Ok(&[]);
        };
        self.original_header(definition)?
            .ok_or("imported input header disappeared")?;
        Ok(if declared {
            &header.inputs[..header.declared]
        } else {
            &header.inputs
        })
    }
}
pub(super) fn import(imported: &Imported, facts: &mut Declarations) -> Result<(), Error> {
    if !imported
        .dependencies()
        .artifacts()
        .iter()
        .any(|a| a.generic_payload().is_some())
    {
        return Ok(());
    }
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
        let rows = payload.rows();
        headers.prepare(rows)?;
        let types = roots::import(imported, facts, artifact.identity(), rows, &mut headers)?;
        for body in &rows.originals {
            for local in &body.locals {
                headers.charge(1)?;
                let Some(reference) = local.definition else {
                    continue;
                };
                let definition = imported
                    .generic_definition(artifact.identity(), reference)
                    .map_err(error)?;
                let slot = facts
                    .definitions
                    .get_mut(definition.0 as usize)
                    .ok_or_else(|| error("imported local has no real declaration slot"))?;
                if slot.replace(types.nodes[local.ty.0 as usize]).is_some() {
                    return Err(error("imported local declaration was already typed"));
                }
            }
        }
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
