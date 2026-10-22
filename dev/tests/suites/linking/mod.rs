//! Maintainer integration suites for linking.
#[path = "../../crate_linking.rs"]
mod crate_linking;
#[path = "../../foreign_conventions.rs"]
mod foreign_conventions;
#[path = "../../foreign_declarations.rs"]
mod foreign_declarations;
#[path = "../../harness/foreign_frontend.rs"]
pub(crate) mod foreign_frontend;
#[path = "../../foreign_linkage.rs"]
mod foreign_linkage;
#[path = "../../harness/host_objects.rs"]
pub(crate) mod host_objects;
#[path = "../../library_nominals.rs"]
mod library_nominals;
#[path = "../../library_roots.rs"]
mod library_roots;
#[path = "../../linkage_symbols.rs"]
mod linkage_symbols;
#[path = "../../metadata_codec.rs"]
mod metadata_codec;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../helpers/resolve.rs"]
pub(crate) mod resolve;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../symbol_definitions.rs"]
mod symbol_definitions;
