//! Imported contracts are remapped once; const execution still requires checked bodies.
use super::{foreign, model::Declarations};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::FileId},
    support::{
        crates::{CrateIdentity, Scalar, Symbol},
        linkage::{Linkage as Kind, Naming},
    },
    syntax::item::Safety,
    types::{GenericArgs, ItemId, Kind as Type, Primitive, TypeId, callable::Convention},
};
use std::collections::HashMap;

#[derive(Default)]
pub struct Linkage {
    identity: Option<CrateIdentity>,
    symbols: HashMap<DefId, String>,
}
impl Linkage {
    #[must_use]
    pub fn rust_symbol(&self, definition: DefId) -> Option<&str> {
        self.symbols.get(&definition).map(String::as_str)
    }
    #[must_use]
    pub const fn identity(&self) -> Option<&CrateIdentity> {
        self.identity.as_ref()
    }
}
fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}
pub(super) fn import(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    let linkage = facts.crates.get_or_insert_default();
    let registry = facts.foreign.get_or_insert_default();
    for (definition, function) in imported.functions() {
        let parameters: Vec<_> = function
            .signature()
            .parameters()
            .iter()
            .copied()
            .map(scalar)
            .collect();
        let parameters = facts.types.intern_list(&parameters).map_err(error)?;
        let signature = facts
            .types
            .intern(Type::Function {
                bound_regions: 0,
                parameters,
                result: scalar(function.signature().result()),
                unsafe_: function.signature().unsafe_(),
                convention: Convention::RUST,
            })
            .map_err(error)?;
        let ty = facts
            .types
            .intern(Type::FunctionItem {
                item: ItemId(definition.0),
                arguments: GenericArgs::EMPTY,
                signature,
            })
            .map_err(error)?;
        facts.definitions[definition.0 as usize] = Some(ty);
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
    Ok(())
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
