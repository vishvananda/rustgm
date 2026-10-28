//! Build source formatting calls around already parsed, separately adopted inputs.
#[path = "format_arguments/emitted_arguments.rs"]
mod output;
#[path = "format_arguments/encoded_pieces.rs"]
mod template;
use super::choice::Charge;
use super::{Config, Expansion, MacroContext};
use crate::{
    lex::token::LexError,
    support::names::{NameId, Names},
    syntax::{ast::NodeId, token_tree::error},
};

pub(crate) struct Draft {
    pub expansion: Expansion,
    /// Local generated occurrence and its actual eager input ordinal.
    pub inputs: Vec<(NodeId, usize)>,
    pub captured: Vec<NodeId>,
}
/// Borrowed call environment; no proof, retained owner or allocation.
pub(crate) struct Environment<'a> {
    pub symbols: &'a mut Names,
    pub grammar: MacroContext,
    pub config: &'a Config,
}
pub(crate) fn build(
    text: &str,
    input_names: &[Option<&str>],
    allow_capture: bool,
    environment: Environment<'_>,
    newline: bool,
    names: [NameId; 4],
    charge: &Charge<'_>,
) -> Result<Draft, LexError> {
    charge(
        1,
        size_of::<output::Output<'_>>() + size_of::<Environment<'_>>(),
    )?;
    let Environment {
        symbols,
        grammar,
        config,
    } = environment;
    if !matches!(grammar, MacroContext::Expression | MacroContext::Statements) {
        return Err(error(0, "formatting requires an expression context"));
    }
    let count = input_names.len();
    let template =
        template::compile_named(text, count, input_names, allow_capture, newline, charge)?;
    output::Output::build(template, count, names, symbols, grammar, config, charge)
}
