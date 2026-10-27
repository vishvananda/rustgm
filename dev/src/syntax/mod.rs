//! Syntax consumes typed tokens. Retained trees belong to captured fragments.
pub mod ast;
pub mod attribute;
pub(crate) mod compound;
pub mod cursor;
pub mod defaultness;
pub mod edition_context;
pub mod expression;
pub mod gate_sites;
pub mod interpolation;
pub mod item;
pub(crate) mod offset_fields;
pub mod token_tree;
pub mod transfer;
mod walk;

pub(crate) mod assembly;
