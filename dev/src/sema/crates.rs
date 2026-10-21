//! Imported contracts are remapped once; const execution still requires checked bodies.
mod export;
mod imported_values;
mod type_transport;
use super::{foreign, model::Declarations};
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
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Linkage {
    identity: Option<CrateIdentity>,
    prefix: Option<String>,
    symbols: HashMap<DefId, String>,
    imported_constants: HashSet<DefId>,
    import_work: ImportWork,
}
/// Exact signature graph and root visits during one source-free import pass.
#[derive(Clone, Copy, Default)]
pub struct ImportWork {
    pub signatures: usize,
    pub type_nodes: usize,
    pub roots: usize,
}
impl Linkage {
    #[must_use]
    pub const fn import_work(&self) -> ImportWork {
        self.import_work
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
pub(super) fn import(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    for (definition, function) in imported.functions() {
        let signature = type_transport::import(facts, function.signature())?;
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
    imported_values::seed(report, facts)
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
