#![feature(no_core)]
#![no_core]
extern crate exporter;
// Re-exports of another crate's definitions and of its re-exports.
pub use exporter::builtin::concat;
pub use exporter::prelude::v1::line as here;
pub mod nested {
    // Through the root `extern crate` item.
    pub use crate::exporter::forty_two;
}
