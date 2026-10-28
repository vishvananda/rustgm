//! Maintainer integration suites for constants.
#[path = "../../harness/checked_sources.rs"]
pub(crate) mod checked_sources;
#[path = "../../const_functions.rs"]
mod const_functions;
#[path = "../../constant_destruction.rs"]
mod constant_destruction;
#[path = "../../constant_layouts.rs"]
mod constant_layouts;
#[path = "../../constant_locations.rs"]
mod constant_locations;
#[path = "../../constant_queries.rs"]
mod constant_queries;
#[path = "../../constant_storage.rs"]
mod constant_storage;
#[path = "../../discriminant_execution.rs"]
mod discriminant_execution;
#[path = "../../discriminant_readiness.rs"]
mod discriminant_readiness;
#[path = "../../discriminants.rs"]
mod discriminants;
#[path = "../../dst_tail_metadata.rs"]
mod dst_tail_metadata;
#[path = "../../evaluated_inspection.rs"]
mod evaluated_inspection;
#[path = "../../initializer_indexing.rs"]
mod initializer_indexing;
#[path = "../../initializer_lifetimes.rs"]
mod initializer_lifetimes;
#[path = "../../initializer_selection.rs"]
mod initializer_selection;
#[path = "../../initializer_validation.rs"]
mod initializer_validation;
#[path = "../../interval_publication.rs"]
mod interval_publication;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../nominal_constants.rs"]
mod nominal_constants;
#[path = "../../product_constants.rs"]
mod product_constants;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../required_values.rs"]
mod required_values;
#[path = "../../helpers/resolve.rs"]
pub(crate) mod resolve;
#[path = "../../rest_views.rs"]
mod rest_views;
#[path = "../../selected_initializers.rs"]
mod selected_initializers;
#[path = "../../support/semantic.rs"]
pub(crate) mod semantic;
#[path = "../../sequence_constants.rs"]
mod sequence_constants;
#[path = "../../helpers/source_cases.rs"]
pub(crate) mod source_cases;

#[path = "../../harness/warning_contracts.rs"]
pub(crate) mod warning_contracts;
