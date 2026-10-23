//! Maintainer integration suites for frontend.
#[path = "../../crate_expansion.rs"]
mod crate_expansion;
#[path = "../../crate_syntax.rs"]
mod crate_syntax;
#[path = "../../edition_fragments.rs"]
mod edition_fragments;
#[path = "../../edition_origins.rs"]
mod edition_origins;
#[path = "../../edition_tokens.rs"]
mod edition_tokens;
#[path = "../../expanded_cli.rs"]
mod expanded_cli;
#[path = "../../expansion_install.rs"]
mod expansion_install;
#[path = "../../expansion_stream.rs"]
mod expansion_stream;
#[path = "../../expressions.rs"]
mod expressions;
#[path = "../../feature_requests.rs"]
mod feature_requests;
#[path = "../../fragments.rs"]
mod fragments;
#[path = "../../hygiene.rs"]
mod hygiene;
#[path = "../../interpolation.rs"]
mod interpolation;
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
#[path = "../../modules.rs"]
mod modules;
#[path = "../../name_storage.rs"]
mod name_storage;
#[path = "../../harness/native_references.rs"]
pub(crate) mod native_references;
#[path = "../../option_environment.rs"]
mod option_environment;
#[path = "../../helpers/qualify.rs"]
pub(crate) mod qualify;
#[path = "../../resolution.rs"]
mod resolution;
#[path = "../../syntax_transfer.rs"]
mod syntax_transfer;
#[path = "../../textual_scopes.rs"]
mod textual_scopes;
#[path = "../../token_trees.rs"]
mod token_trees;
