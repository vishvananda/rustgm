//! Build source formatting calls around already parsed, separately adopted inputs.
#[path = "format_arguments/emitted_arguments.rs"]
mod output;
#[path = "format_arguments/encoded_pieces.rs"]
mod template;
use super::choice::Charge;
use super::{Config, Expansion, MacroContext};
use crate::{
    lex::token::LexError,
    support::names::NameId,
    syntax::{ast::NodeId, token_tree::error},
};

pub(crate) struct Draft {
    pub expansion: Expansion,
    /// Local generated occurrence and its actual eager input ordinal.
    pub inputs: Vec<(NodeId, usize)>,
}
pub(crate) fn build(
    text: &str,
    count: usize,
    newline: bool,
    names: [NameId; 3],
    grammar: MacroContext,
    config: &Config,
    charge: &Charge<'_>,
) -> Result<Draft, LexError> {
    charge(1, size_of::<output::Output<'_>>())?;
    if !matches!(grammar, MacroContext::Expression | MacroContext::Statements) {
        return Err(error(0, "formatting requires an expression context"));
    }
    let template = template::compile(text, count, newline, charge)?;
    output::Output::build(template, count, names, grammar, config, charge)
}
