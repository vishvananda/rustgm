//! Stable external identities and bounded scalar metadata never contain local arena IDs.
mod codec;
mod identity;
mod interface;
pub use codec::{SCHEMA, decode, encode};
pub use identity::{CrateIdentity, ExternalDef, ExternalKind, Namespace};
pub use interface::{Function, Interface, Scalar, Signature, Symbol};

pub(crate) const MAX_BYTES: usize = 16_000_000;
pub(crate) const MAX_FUNCTIONS: usize = 100_000;
pub(crate) const MAX_DEPENDENCIES: usize = 1024;
pub(crate) const MAX_COMPONENTS: usize = 1_000_000;
pub(crate) const MAX_NAME_BYTES: usize = 8_000_000;
