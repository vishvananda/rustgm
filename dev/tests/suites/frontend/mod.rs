//! Maintainer integration suites for frontend.
#[path = "../../anon_borrows.rs"]
mod anon_borrows;
#[path = "../../applicable_contracts.rs"]
mod applicable_contracts;
#[path = "../../arbitrary_methods.rs"]
mod arbitrary_methods;
#[path = "../../async_signatures.rs"]
mod async_signatures;
#[path = "../../asynccleanup.rs"]
mod asynccleanup;
#[path = "../../automatic_sources.rs"]
mod automatic_sources;
#[path = "../../callback_addresses.rs"]
mod callback_addresses;
#[path = "../../callback_provider.rs"]
mod callback_provider;
#[path = "../../capped_cli.rs"]
mod capped_cli;
#[path = "../../cell_sources.rs"]
mod cell_sources;
#[path = "../../closureconst.rs"]
mod closureconst;
#[path = "../../coerce_validation_source.rs"]
mod coerce_validation_source;
#[path = "../../coercion_provider.rs"]
mod coercion_provider;
#[path = "../../configured_tails.rs"]
mod configured_tails;
#[path = "../../crate_expansion.rs"]
mod crate_expansion;
#[path = "../../crate_syntax.rs"]
mod crate_syntax;
#[path = "../../creator_inputs.rs"]
mod creator_inputs;
#[path = "../../cstr_sources.rs"]
mod cstr_sources;
#[path = "../../ctfe_selector.rs"]
mod ctfe_selector;
#[path = "../../declarative_macros.rs"]
mod declarative_macros;
#[path = "../../deref_contracts.rs"]
mod deref_contracts;
#[path = "../../derived_field_bounds.rs"]
mod derived_field_bounds;
#[path = "../../edition_fragments.rs"]
mod edition_fragments;
#[path = "../../edition_origins.rs"]
mod edition_origins;
#[path = "../../edition_packages.rs"]
mod edition_packages;
#[path = "../../edition_tokens.rs"]
mod edition_tokens;
#[path = "../../ergonomic_contracts.rs"]
mod ergonomic_contracts;
#[path = "../../expanded_cli.rs"]
mod expanded_cli;
#[path = "../../expansion_install.rs"]
mod expansion_install;
#[path = "../../expansion_stream.rs"]
mod expansion_stream;
#[path = "../../expressions.rs"]
mod expressions;
#[path = "../../extern_editions.rs"]
mod extern_editions;
#[path = "../../feature_requests.rs"]
mod feature_requests;
#[path = "../../helpers/ffi_program.rs"]
pub(crate) mod ffi_program;
#[path = "../../forced_dyn_contracts.rs"]
mod forced_dyn_contracts;
#[path = "../../fragments.rs"]
mod fragments;
#[path = "../../from_yeet_sources.rs"]
mod from_yeet_sources;
#[path = "../../fundamental_contracts.rs"]
mod fundamental_contracts;
#[path = "../../fused_sources.rs"]
mod fused_sources;
#[path = "../../future_helpers.rs"]
mod future_helpers;
#[path = "../../future_source.rs"]
mod future_source;
#[path = "../../glob_provenance.rs"]
mod glob_provenance;
#[path = "../../glue_provider.rs"]
mod glue_provider;
#[path = "../../hygiene.rs"]
mod hygiene;
#[path = "../../integer_products.rs"]
mod integer_products;
#[path = "../../interpolation.rs"]
mod interpolation;
#[path = "../../interval_sources.rs"]
mod interval_sources;
#[path = "../../into_future.rs"]
mod into_future;
#[path = "../../inventory_ordering.rs"]
mod inventory_ordering;
#[path = "../../keyed_parameters.rs"]
mod keyed_parameters;
#[path = "../../legacy_editions.rs"]
mod legacy_editions;
#[path = "../../lending_calls.rs"]
mod lending_calls;
#[path = "../../lexer.rs"]
mod lexer;
#[path = "../../literal_values.rs"]
mod literal_values;
#[path = "../../macro_forwarding.rs"]
mod macro_forwarding;
#[path = "../../macro_matching.rs"]
mod macro_matching;
#[path = "../../macro_registry.rs"]
mod macro_registry;
#[path = "../../macro_rules.rs"]
mod macro_rules;
#[path = "../../macro_session.rs"]
mod macro_session;
#[path = "../../macro_transcription.rs"]
mod macro_transcription;
#[path = "../../matchability.rs"]
mod matchability;
#[path = "../../modular_primitives.rs"]
mod modular_primitives;
#[path = "../../modules.rs"]
mod modules;
#[path = "../../name_storage.rs"]
mod name_storage;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../harness/observed_program.rs"]
pub(crate) mod observed_program;
#[path = "../../option_environment.rs"]
mod option_environment;
#[path = "../../pattern_editions.rs"]
mod pattern_editions;
#[path = "../../pinned_drop.rs"]
mod pinned_drop;
#[path = "../../pinning_contracts.rs"]
mod pinning_contracts;
#[path = "../../polling_source.rs"]
mod polling_source;
#[path = "../../prototype_contracts.rs"]
mod prototype_contracts;
#[path = "../../provider_traits.rs"]
mod provider_traits;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../quota_options.rs"]
mod quota_options;
#[path = "../../record_availability.rs"]
mod record_availability;
#[path = "../../reflection_sources.rs"]
mod reflection_sources;
#[path = "../../resolution.rs"]
mod resolution;
#[path = "../../helpers/resolve.rs"]
pub(crate) mod resolve;
#[path = "../../resuming_sources.rs"]
mod resuming_sources;
#[path = "../../rhs_body_lifetimes.rs"]
mod rhs_body_lifetimes;
#[path = "../../selected_references.rs"]
mod selected_references;
#[path = "../../smart_receivers.rs"]
mod smart_receivers;
#[path = "../../stateframe.rs"]
mod stateframe;
#[path = "../../syntax_transfer.rs"]
mod syntax_transfer;
#[path = "../../target_requests.rs"]
mod target_requests;
#[path = "../../temporary_editions.rs"]
mod temporary_editions;
#[path = "../../textual_scopes.rs"]
mod textual_scopes;
#[path = "../../token_trees.rs"]
mod token_trees;
#[path = "../../tuple_index_privacy.rs"]
mod tuple_index_privacy;
#[path = "../../zero_sized_niche_companions.rs"]
mod zero_sized_niche_companions;

#[path = "../../needs_drop_provider.rs"]
mod needs_drop_provider;

#[path = "../../never_editions.rs"]
mod never_editions;

#[path = "../../availability_contracts.rs"]
mod availability_contracts;
#[path = "../../return_editions.rs"]
mod return_editions;

#[path = "../../frontend_diagnostics.rs"]
mod frontend_diagnostics;

#[path = "../../attribute_contracts.rs"]
mod attribute_contracts;

#[path = "../../diagnostic_contracts.rs"]
mod diagnostic_contracts;

#[path = "../../discarded_domains.rs"]
mod discarded_domains;

#[path = "../../inhabited_aggregates.rs"]
mod inhabited_aggregates;

#[path = "../../test_removal_lints.rs"]
mod test_removal_lints;

#[path = "../../wrapper_sources.rs"]
mod wrapper_sources;

#[path = "../../group_execution.rs"]
mod group_execution;

#[path = "../../access_sources.rs"]
mod access_sources;
#[path = "../../stream_sources.rs"]
mod stream_sources;

#[path = "../../autoref_lints.rs"]
mod autoref_lints;

#[path = "../../coroutine_sources.rs"]
mod coroutine_sources;

#[path = "../../overload_captures.rs"]
mod overload_captures;

#[path = "../../assertion_sources.rs"]
mod assertion_sources;

#[path = "../../qualified_types.rs"]
mod qualified_types;
#[path = "../../reborrow_sources.rs"]
mod reborrow_sources;
#[path = "../../weak_symbols.rs"]
mod weak_symbols;

#[path = "../../overflow_words.rs"]
mod overflow_words;

#[path = "../../cold_sources.rs"]
mod cold_sources;

#[path = "../../guaranteed_pointers.rs"]
mod guaranteed_pointers;

#[path = "../../cold_prototypes.rs"]
mod cold_prototypes;

#[path = "../../deprecation_releases.rs"]
mod deprecation_releases;

#[path = "../../census_mode.rs"]
mod census_mode;

#[path = "../../knowledge_sources.rs"]
mod knowledge_sources;

#[path = "../../loop_results.rs"]
mod loop_results;

#[path = "../../interchange_images.rs"]
mod interchange_images;
