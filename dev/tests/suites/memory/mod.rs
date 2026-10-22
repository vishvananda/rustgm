//! Maintainer integration suites for memory.
#[path = "../../atomic_storage.rs"]
mod atomic_storage;
#[path = "../../bitwise_atomics.rs"]
mod bitwise_atomics;
#[path = "../../harness/bitwise_loops.rs"]
pub(crate) mod bitwise_loops;
#[path = "../../byte_fills.rs"]
mod byte_fills;
#[path = "../../byte_transfers.rs"]
mod byte_transfers;
#[path = "../../cas_intrinsics.rs"]
mod cas_intrinsics;
#[path = "../../harness/checked_sources.rs"]
pub(crate) mod checked_sources;
#[path = "../../harness/counted_cases.rs"]
pub(crate) mod counted_cases;
#[path = "../../counted_transfers.rs"]
mod counted_transfers;
#[path = "../../direct_union_borrows.rs"]
mod direct_union_borrows;
#[path = "../../external_storage.rs"]
mod external_storage;
#[path = "../../extremum_atomics.rs"]
mod extremum_atomics;
#[path = "../../finite_storage.rs"]
mod finite_storage;
#[path = "../../harness/foreign_frontend.rs"]
pub(crate) mod foreign_frontend;
#[path = "../../harness/host_objects.rs"]
pub(crate) mod host_objects;
#[path = "../../image_storage.rs"]
mod image_storage;
#[path = "../../intrinsic_backend.rs"]
mod intrinsic_backend;
#[path = "../../intrinsic_methods.rs"]
mod intrinsic_methods;
#[path = "../../memory_transfers.rs"]
mod memory_transfers;
#[path = "../../mutable_reconstruction.rs"]
mod mutable_reconstruction;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../overlapping_transfers.rs"]
mod overlapping_transfers;
#[path = "../../owned_transfers.rs"]
mod owned_transfers;
#[path = "../../pointer_addresses.rs"]
mod pointer_addresses;
#[path = "../../pointer_casts.rs"]
mod pointer_casts;
#[path = "../../pointer_coercions.rs"]
mod pointer_coercions;
#[path = "../../pointer_constants.rs"]
mod pointer_constants;
#[path = "../../pointer_metadata.rs"]
mod pointer_metadata;
#[path = "../../provenance_reads.rs"]
mod provenance_reads;
#[path = "../../provenance_writes.rs"]
mod provenance_writes;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../reachable_storage.rs"]
mod reachable_storage;
#[path = "../../harness/reconstruction_work.rs"]
pub(crate) mod reconstruction_work;
#[path = "../../reference_access.rs"]
mod reference_access;
#[path = "../../reference_alignment.rs"]
mod reference_alignment;
#[path = "../../reference_cells.rs"]
mod reference_cells;
#[path = "../../reference_reconstruction.rs"]
mod reference_reconstruction;
#[path = "../../relocation_transport.rs"]
mod relocation_transport;
#[path = "../../helpers/resolve.rs"]
pub(crate) mod resolve;
#[path = "../../rmw_intrinsics.rs"]
mod rmw_intrinsics;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../helpers/source_cases.rs"]
pub(crate) mod source_cases;
#[path = "../../static_addresses.rs"]
mod static_addresses;
#[path = "../../subtract_intrinsics.rs"]
mod subtract_intrinsics;
#[path = "../../unaligned_transfers.rs"]
mod unaligned_transfers;
#[path = "../../union_addresses.rs"]
mod union_addresses;
#[path = "../../union_references.rs"]
mod union_references;
#[path = "../../unsafe_contexts.rs"]
mod unsafe_contexts;
#[path = "../../unsafe_policies.rs"]
mod unsafe_policies;
#[path = "../../unsafe_policy_modules.rs"]
mod unsafe_policy_modules;
#[path = "../../unsafe_precision.rs"]
mod unsafe_precision;
#[path = "../../unsafe_statics.rs"]
mod unsafe_statics;
#[path = "../../unsafe_unions.rs"]
mod unsafe_unions;
#[path = "../../volatile_intrinsics.rs"]
mod volatile_intrinsics;
