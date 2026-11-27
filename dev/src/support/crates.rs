//! Stable external identities and bounded scalar metadata never contain local arena IDs.
mod artifact_policy;
mod codec;
mod dependencies;
pub mod exported_macros;
pub mod generics;
mod globals;
mod guest;
mod identity;
mod interface;
mod nominals;
pub mod owner_catalog;
mod public_names;
mod retirements;
mod scalar_value;
mod type_graph;
pub use artifact_policy::{ArtifactContext, OutputKind};
pub use codec::{SCHEMA, decode, encode};
pub use dependencies::{Dependencies, Dependency};
pub use generics::source::{Descriptor as SourceDescriptor, Profile as SourceProfile};
pub use globals::{
    Static, StaticRef, Storage, StorageOrigin, StoredKind, StoredProjection, StoredValue, ValueRef,
};
pub use guest::Guest;
pub use identity::{CrateIdentity, ExternalDef, ExternalKind, Namespace};
pub use interface::{Function, Interface, LoaderFlags, Scalar, Symbol};
pub use nominals::{Nominals, Record, RecordField, RecordRef, RecordRepr};
pub use public_names::{PublicBinding, PublicModule, PublicNames};
pub use retirements::{RemovedFeature, RemovedFeatures, RemovedVersion};
pub(crate) use scalar_value::ScalarLiteral;
pub use scalar_value::{Constant, ScalarValue};
pub use type_graph::{Abi, Lifetime, Signature, TypeGraph, TypeNode, TypeRef};

pub(crate) const MAX_BYTES: usize = 1 << 30;
/// The encoded artifact byte ceiling, shared by the loader and the codec.
pub const MAX_ARTIFACT_BYTES: usize = MAX_BYTES;
pub(crate) const MAX_FUNCTIONS: usize = 1_048_576;
pub(crate) const MAX_DEPENDENCIES: usize = 1024;
pub(crate) const MAX_COMPONENTS: usize = 67_108_864;
pub(crate) const MAX_NAME_BYTES: usize = 67_108_864;
