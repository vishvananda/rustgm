//! Maintainer integration suites for regions.
#[path = "../../caller_outlives.rs"]
mod caller_outlives;
#[path = "../../defining_regions.rs"]
mod defining_regions;
#[path = "../../field_outlives.rs"]
mod field_outlives;
#[path = "../../instance_regions.rs"]
mod instance_regions;
#[path = "../../lifetime_declarations.rs"]
mod lifetime_declarations;
#[path = "../../lifetime_substitution.rs"]
mod lifetime_substitution;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../nominal_outlives.rs"]
mod nominal_outlives;
#[path = "../../opaque_outlives.rs"]
mod opaque_outlives;
#[path = "../../outlives_types.rs"]
mod outlives_types;
#[path = "../../outlives_verification.rs"]
mod outlives_verification;
#[path = "../../projection_lifetimes.rs"]
mod projection_lifetimes;
#[path = "../../projection_region_mapping.rs"]
mod projection_region_mapping;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../quantified_lifetime_proofs.rs"]
mod quantified_lifetime_proofs;
#[path = "../../region_constraints.rs"]
mod region_constraints;
#[path = "../../region_graph.rs"]
mod region_graph;
#[path = "../../region_inference.rs"]
mod region_inference;
#[path = "../../region_points.rs"]
mod region_points;
#[path = "../../result_lifetimes.rs"]
mod result_lifetimes;
#[path = "../../result_liveness.rs"]
mod result_liveness;
#[path = "../../return_regions.rs"]
mod return_regions;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../symbolic_outlives.rs"]
mod symbolic_outlives;
#[path = "../../symbolic_outlives_source.rs"]
mod symbolic_outlives_source;
#[path = "../../template_regions.rs"]
mod template_regions;
