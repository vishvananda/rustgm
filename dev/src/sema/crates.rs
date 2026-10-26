//! Imported contracts are remapped once; const execution still requires checked bodies.
pub(crate) mod authored_export;
mod export;
#[path = "crates/checked_ownership.rs"]
mod imported_owners;
mod imported_records;
mod imported_storage;
mod imported_values;
mod type_transport;
mod typed_headers;
use super::{foreign, model::Declarations};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::FileId},
    support::{
        crates::{CrateIdentity, Scalar, Symbol},
        linkage::{Linkage as Kind, Naming},
    },
    syntax::item::Safety,
    types::{GenericArgs, ItemId, Kind as Type, Primitive, TypeId},
};
pub use export::{Exports, export};
pub(crate) use export::{GenericRequest, export_with_originals, scalar as export_scalar};
pub(in crate::sema) use imported_owners::ImportedOwners;
pub(in crate::sema) use typed_headers::predicate_remap::Prepared as PreparedSourceInputs;
pub(super) use typed_headers::{
    charge_source_headers, mapped_source::Readiness as ImportedReadiness, source_readiness,
    source_trait_header,
};
pub(crate) use typed_headers::{
    is_source_original, mapped_source::SourceOriginal, source_original,
};

#[derive(Default)]
pub struct Linkage {
    origins: Option<std::rc::Rc<crate::resolve::body_origin::Catalog>>,
    originals: Option<Box<typed_headers::Headers>>,
    identity: Option<CrateIdentity>,
    prefix: Option<String>,
    symbols: HashMap<DefId, String>,
    imported_constants: HashSet<DefId>,
    imported_records: HashSet<DefId>,
    imported_storage: HashMap<DefId, bool>,
    import_work: ImportWork,
}
/// Exact signature graph and root visits during one source-free import pass.
#[derive(Clone, Copy, Default)]
pub struct ImportWork {
    pub signatures: usize,
    pub type_nodes: usize,
    pub roots: usize,
    pub records: usize,
    pub fields: usize,
}
impl Linkage {
    pub(crate) fn origin_catalog(&self) -> Option<&crate::resolve::body_origin::Catalog> {
        self.origins.as_deref()
    }
    /// Imported Rust storage has completed contents; foreign declarations do not.
    #[must_use]
    pub fn imported_storage(&self, definition: DefId) -> Option<bool> {
        self.imported_storage.get(&definition).copied()
    }
    #[must_use]
    pub const fn import_work(&self) -> ImportWork {
        self.import_work
    }
    pub(crate) fn imported_record(&self, definition: DefId) -> bool {
        self.imported_records.contains(&definition)
    }
    /// Install one explicit local identity without changing imported contracts.
    /// # Errors
    /// Rejects a different identity after naming has been published.
    pub fn set_identity(&mut self, identity: CrateIdentity) -> Result<(), &'static str> {
        if self.identity.as_ref().is_some_and(|old| old != &identity) {
            return Err("conflicting local crate linkage identity");
        }
        if self.identity.is_none() {
            self.prefix = Some(identity.symbol_prefix());
            self.identity = Some(identity);
        }
        Ok(())
    }
    #[must_use]
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    #[must_use]
    pub fn rust_symbol(&self, definition: DefId) -> Option<&str> {
        self.symbols.get(&definition).map(String::as_str)
    }
    #[must_use]
    pub const fn identity(&self) -> Option<&CrateIdentity> {
        self.identity.as_ref()
    }
    #[must_use]
    pub fn imported_constant(&self, definition: DefId) -> bool {
        self.imported_constants.contains(&definition)
    }
}
fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}
pub(in crate::sema) fn reserve_staged(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let actual = report
        .package
        .staged_consumer()
        .ok_or_else(|| error("private semantic reservation requires its actual consumer policy"))?;
    actual.require_package(&report.package).map_err(error)?;
    let origins = report
        .resolution
        .crates
        .as_deref()
        .ok_or_else(|| error("private semantic reservation lacks its actual imported inventory"))?
        .origin_catalog()
        .map_err(error)?
        .ok_or_else(|| error("private semantic reservation lacks its actual shared catalog"))?;
    if facts.crates.is_some() {
        return Err(error(
            "private semantic reservation cannot replace installed linkage",
        ));
    }
    facts.crates = Some(Box::new(Linkage {
        origins: Some(origins),
        ..Linkage::default()
    }));
    Ok(())
}
pub(in crate::sema) const fn staged_linkage_allocation() -> usize {
    size_of::<Linkage>()
}
pub(super) fn import(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    if let Some(origins) = imported.origin_catalog().map_err(error)? {
        facts.crates.get_or_insert_default().origins = Some(origins);
    }
    let records = imported_records::prepare(imported, facts)?;
    typed_headers::import(report, imported, facts)?;
    for (definition, function) in imported.functions() {
        let signature = records.signature(facts, function)?;
        let ty = facts
            .types
            .intern(Type::FunctionItem {
                item: ItemId(definition.0),
                arguments: GenericArgs::EMPTY,
                signature,
            })
            .map_err(error)?;
        facts.definitions[definition.0 as usize] = Some(ty);
        let linkage = facts.crates.get_or_insert_default();
        let registry = facts.foreign.get_or_insert_default();
        let naming = match function.symbol() {
            Symbol::Rust => {
                linkage
                    .symbols
                    .insert(definition, function.definition().rust_symbol());
                Naming::Rust
            }
            Symbol::Exact(name) => Naming::Exact(registry.names_mut().intern(name).map_err(error)?),
        };
        registry
            .insert_function(
                definition,
                foreign::Function {
                    signature,
                    safety: if function.signature().unsafe_() {
                        Safety::Unsafe
                    } else {
                        Safety::Default
                    },
                    naming,
                    linkage: Kind::Import,
                },
            )
            .map_err(error)?;
    }
    imported_values::seed(report, facts)?;
    imported_storage::seed(report, facts, &records)
}
pub(super) const fn scalar(value: Scalar) -> TypeId {
    TypeId::primitive(match value {
        Scalar::Unit => return TypeId::UNIT,
        Scalar::Bool => Primitive::Bool,
        Scalar::Char => Primitive::Char,
        Scalar::U8 => Primitive::U8,
        Scalar::U16 => Primitive::U16,
        Scalar::U32 => Primitive::U32,
        Scalar::U64 => Primitive::U64,
        Scalar::U128 => Primitive::U128,
        Scalar::Usize => Primitive::Usize,
        Scalar::I8 => Primitive::I8,
        Scalar::I16 => Primitive::I16,
        Scalar::I32 => Primitive::I32,
        Scalar::I64 => Primitive::I64,
        Scalar::I128 => Primitive::I128,
        Scalar::Isize => Primitive::Isize,
        Scalar::F32 => Primitive::F32,
        Scalar::F64 => Primitive::F64,
    })
}
