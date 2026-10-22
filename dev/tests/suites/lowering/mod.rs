//! Maintainer integration suites for lowering.
#[path = "../../aggregate_calling.rs"]
mod aggregate_calling;
#[path = "../../aggregate_lowering.rs"]
mod aggregate_lowering;
#[path = "../../body_types.rs"]
mod body_types;
#[path = "../../cfg_execution.rs"]
mod cfg_execution;
#[path = "../../constructor_bodies.rs"]
mod constructor_bodies;
#[path = "../../harness/foreign_frontend.rs"]
pub(crate) mod foreign_frontend;
#[cfg(feature = "object-interop-tests")]
#[path = "../../harness/host_objects.rs"]
pub(crate) mod host_objects;
#[path = "../../instance_bodies.rs"]
mod instance_bodies;
#[path = "../../instance_constructors.rs"]
mod instance_constructors;
#[path = "../../instance_destructors.rs"]
mod instance_destructors;
#[path = "../../instance_values.rs"]
mod instance_values;
#[path = "../../lowir_data.rs"]
mod lowir_data;
#[path = "../../lowir_lowering.rs"]
mod lowir_lowering;
#[path = "../../native_generic_aliases.rs"]
mod native_generic_aliases;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../object_dispatch.rs"]
mod object_dispatch;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../thin_references.rs"]
mod thin_references;
#[path = "../../typed_bodies.rs"]
mod typed_bodies;
#[path = "../../wide_data.rs"]
mod wide_data;
#[path = "../../wide_references.rs"]
mod wide_references;
