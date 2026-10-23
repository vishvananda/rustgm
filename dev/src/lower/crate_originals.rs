//! The exporter consumes actual Checked originals, never a reconstructed body.
mod declarations;
mod formal_links;
mod lexical;
mod ports;
mod signatures;
use crate::{
    body::{
        ir::{
            Action, Body, BodyOrigin, BodySite, DefId, LocalId, LocalKind, NodeKey, Owner, Place,
            Rvalue, Terminator, ValueId,
        },
        regions::Contract,
    },
    borrow::templates::Checked,
    resolve::{expansion::Report, model::DefKind, package::Error},
    sema::{crates::GenericRequest, model::Declarations},
    support::{
        crates::{
            self, Interface, generics as g,
            owner_catalog::{BodyRef, DeclarationRef},
        },
        hash::HashMap,
    },
    types::{ItemId, TypeId},
};

struct Export<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    rows: g::Rows,
    declarations: HashMap<DefId, DeclarationRef>,
    types: HashMap<TypeId, g::TypeRef>,
    formals: HashMap<DefId, (u32, u32, u32)>,
    steps: usize,
    bytes: usize,
}
impl Export<'_> {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > crates::MAX_COMPONENTS {
            return Err("generic original export work budget exceeded");
        }
        Ok(())
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.charge(1)?;
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > crates::MAX_BYTES {
            return Err("generic original export storage budget exceeded");
        }
        Ok(())
    }
    fn ordinal(len: usize) -> Result<u32, &'static str> {
        u32::try_from(len).map_err(|_| "generic export table overflow")
    }
    fn declaration(&self, id: DefId) -> Result<DeclarationRef, &'static str> {
        self.declarations
            .get(&id)
            .copied()
            .ok_or("exported body names an unreserved actual declaration")
    }
}
pub(super) fn export(
    report: &Report,
    facts: &Declarations,
    checked: &Checked<'_>,
    requests: &[GenericRequest],
    base: &Interface,
) -> Result<g::Payload, Error> {
    let error = |e| report.body_optional_error(None, e);
    if requests.is_empty() || facts.uses_source_copy() {
        return Err(error(
            "generic row version zero requires type-generic legacy-role originals",
        ));
    }
    let edition = report
        .package
        .units
        .first()
        .ok_or_else(|| error("generic export requires its defining source unit"))?
        .edition();
    if edition != crate::support::edition::Edition::Rust2024 {
        return Err(error(
            "generic row version zero requires the Rust 2024 source profile",
        ));
    }
    let mut export = Export {
        report,
        facts,
        rows: g::Rows {
            edition,
            declarations: Vec::new(),
            declaration_sites: Vec::new(),
            binders: Vec::new(),
            types: Vec::new(),
            roots: Vec::new(),
            parameters: Vec::new(),
            clauses: Vec::new(),
            functions: Vec::new(),
            originals: Vec::new(),
        },
        declarations: HashMap::default(),
        types: HashMap::default(),
        formals: HashMap::default(),
        steps: 0,
        bytes: size_of::<Export<'_>>(),
    };
    declarations::reserve(&mut export, checked, requests).map_err(error)?;
    for (ordinal, request) in requests.iter().enumerate() {
        let (template, input, _) = checked
            .function_proof(ItemId(request.definition.0))
            .ok_or_else(|| error("generic export requires its actual complete original proof"))?;
        signatures::validate(
            &export,
            request,
            &template.body,
            input,
            &template.environment,
        )
        .map_err(error)?;
        let body = BodyRef(Export::ordinal(ordinal).map_err(error)?);
        let function = signatures::emit(&mut export, request, body).map_err(error)?;
        let original =
            ports::emit(&mut export, &template.body, body, function.declaration).map_err(error)?;
        export
            .reserve(4 * (size_of::<g::Function>() + size_of::<g::Original>()))
            .map_err(error)?;
        export.rows.functions.push(function);
        export.rows.originals.push(original);
    }
    let mut work = g::Work {
        steps: export.steps,
        ..g::Work::default()
    };
    g::Payload::new(base.identity(), export.rows, &mut work).map_err(error)
}
