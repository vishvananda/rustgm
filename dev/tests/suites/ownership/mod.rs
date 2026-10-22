//! Maintainer integration suites for ownership.
#[path = "../../binding_contracts.rs"]
mod binding_contracts;
#[path = "../../borrow_places.rs"]
mod borrow_places;
#[path = "../../helpers/cleanup_lowering.rs"]
pub(crate) mod cleanup_lowering;
#[path = "../../deferred_promotions.rs"]
mod deferred_promotions;
#[path = "../../demanded_cleanup.rs"]
mod demanded_cleanup;
#[path = "../../derived_permissions.rs"]
mod derived_permissions;
#[path = "../../destruction_flags.rs"]
mod destruction_flags;
#[path = "../../destruction_glue.rs"]
mod destruction_glue;
#[path = "../../destruction_plans.rs"]
mod destruction_plans;
#[path = "../../destruction_requirements.rs"]
mod destruction_requirements;
#[path = "../../helpers/destructor_cases.rs"]
pub(crate) mod destructor_cases;
#[path = "../../destructor_conformance.rs"]
mod destructor_conformance;
#[path = "../../destructor_instances.rs"]
mod destructor_instances;
#[path = "../../destructor_loans.rs"]
mod destructor_loans;
#[path = "../../destructor_readiness.rs"]
mod destructor_readiness;
#[path = "../../destructor_regions.rs"]
mod destructor_regions;
#[path = "../../drop_array_partitions.rs"]
mod drop_array_partitions;
#[path = "../../drop_field_moves.rs"]
mod drop_field_moves;
#[path = "../../drop_headers.rs"]
mod drop_headers;
#[path = "../../drop_identity.rs"]
mod drop_identity;
#[path = "../../drop_initialization.rs"]
mod drop_initialization;
#[path = "../../drop_value_scopes.rs"]
mod drop_value_scopes;
#[path = "../../loans.rs"]
mod loans;
#[path = "../../moved_cleanup.rs"]
mod moved_cleanup;
#[path = "../../native_destructors.rs"]
mod native_destructors;
#[path = "../../native_drop_order.rs"]
mod native_drop_order;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../owned_cleanup.rs"]
mod owned_cleanup;
#[path = "../../owned_operands.rs"]
mod owned_operands;
#[path = "../../ownership.rs"]
mod ownership;
#[path = "../../place_mentions.rs"]
mod place_mentions;
#[path = "../../promotions.rs"]
mod promotions;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../reborrows.rs"]
mod reborrows;
#[path = "../../helpers/resolve.rs"]
pub(crate) mod resolve;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../storage_liveness.rs"]
mod storage_liveness;
#[path = "../../symbolic_destruction.rs"]
mod symbolic_destruction;
#[path = "../../symbolic_drop_regions.rs"]
mod symbolic_drop_regions;
#[path = "../../symbolic_operands.rs"]
mod symbolic_operands;
#[path = "../../helpers/template_check.rs"]
pub(crate) mod template_check;
#[path = "../../template_cleanup.rs"]
mod template_cleanup;
#[path = "../../template_loans.rs"]
mod template_loans;
#[path = "../../template_ownership.rs"]
mod template_ownership;
#[path = "../../temporary_scopes.rs"]
mod temporary_scopes;
#[path = "../../value_liveness.rs"]
mod value_liveness;
