//! Stable external identities and bounded scalar metadata never contain local arena IDs.
mod codec;
mod dependencies;
pub mod generics;
mod globals;
mod identity;
mod interface;
mod nominals;
pub mod owner_catalog;
mod public_names;
mod scalar_value;
mod type_graph;
pub use codec::{SCHEMA, decode, encode};
pub use dependencies::{Dependencies, Dependency};
pub use generics::source::{Descriptor as SourceDescriptor, Profile as SourceProfile};
pub use globals::{
    Static, StaticRef, Storage, StorageOrigin, StoredKind, StoredProjection, StoredValue, ValueRef,
};
pub use identity::{CrateIdentity, ExternalDef, ExternalKind, Namespace};
pub use interface::{Function, Interface, Scalar, Symbol};
pub use nominals::{Nominals, Record, RecordField, RecordRef, RecordRepr};
pub use public_names::{PublicBinding, PublicModule, PublicNames};
pub use scalar_value::{Constant, ScalarValue};
pub use type_graph::{Abi, Lifetime, Signature, TypeGraph, TypeNode, TypeRef};

pub(crate) const MAX_BYTES: usize = 16_000_000;
pub(crate) const MAX_FUNCTIONS: usize = 100_000;
pub(crate) const MAX_DEPENDENCIES: usize = 1024;
pub(crate) const MAX_COMPONENTS: usize = 1_000_000;
pub(crate) const MAX_NAME_BYTES: usize = 8_000_000;
