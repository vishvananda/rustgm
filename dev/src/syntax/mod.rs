//! Syntax consumes typed tokens. Retained trees belong to captured fragments.
pub mod ast;
pub mod attribute;
pub mod cursor;
pub mod edition_context;
pub mod expression;
pub mod interpolation;
pub mod item;
pub mod token_tree;
pub mod transfer;
mod walk;
