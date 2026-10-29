//! The source `pattern_type` producer consumes one real typed base fragment.
use super::choice::{Charge, Counted, Selected};
use crate::{
    expand::macros::input::Input,
    lex::token::LexError,
    syntax::{ast::MacroContext, interpolation::Inputs, token_tree::error},
};
pub(crate) fn select<'s>(
    source: &'s str,
    input: &'s Input,
    grammar: MacroContext,
    inputs: &dyn Inputs,
    charge: &Charge<'_>,
) -> Result<Selected<'s>, LexError> {
    let start = input.tokens.first().map_or(0, |token| token.start);
    if grammar != MacroContext::Type {
        return Err(error(start, "pattern_type requires a type context"));
    }
    charge(
        128 + input.tokens.len(),
        input.tokens.len().saturating_mul(256).saturating_add(
            16 * size_of::<crate::syntax::ast::Node>()
                + crate::syntax::gate_sites::endpoint_overhead(),
        ),
    )?;
    let ast = crate::syntax::expression::fragment::type_pattern_with_inputs(
        source,
        &input.tokens,
        &Counted { inputs, charge },
    )?;
    let span = ast.node(ast.root).span;
    Ok(Selected {
        ast,
        span,
        parser_calls: 1,
        cfg_uses: Vec::new(),
    })
}
