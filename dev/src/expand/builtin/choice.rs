//! A real `cfg_select` parses all categories, but returns only the selected syntax.
#[path = "choice_inputs.rs"]
mod inputs;
use crate::{
    expand::{
        config::{self, Config},
        macros::input::Input,
    },
    lex::token::{Kind as TokenKind, LexError, Punctuation as P},
    syntax::{
        ast::{Ast, MacroContext},
        cursor::TokenPosition,
        expression::fragment,
        interpolation::Inputs,
        token_tree::{ForestBuilder, Span, error},
    },
};
pub(crate) use inputs::Charge;
pub(crate) use inputs::{Counted, syntax_payload};
use std::ops::Range;

/// Actual pending-call identity; the resolver supplies its authenticated role separately.
#[derive(Clone, Copy)]
pub(crate) struct Site {
    pub file: crate::support::source::FileId,
    pub target: crate::syntax::ast::NodeId,
    pub original: Span,
    pub grammar: MacroContext,
}
pub(crate) struct Installed {
    pub generated: Span,
    pub parser_calls: usize,
}

struct Branch {
    body: Range<usize>,
    span: Span,
    framed: Option<usize>,
}
#[derive(Default)]
struct Gathered<'s> {
    branches: Vec<Branch>,
    expressions: Vec<Option<Ast<'s>>>,
    first: Option<usize>,
    wildcard: Option<usize>,
    framing_calls: usize,
    cfg_uses: Vec<config::CfgUse>,
}
pub(crate) struct Selected<'s> {
    pub ast: Ast<'s>,
    pub span: Span,
    pub parser_calls: usize,
    pub cfg_uses: Vec<config::CfgUse>,
}
/// The caller owns the real source, token registry, edition view and work budget.
/// No inactive AST is published, configured or expanded by this function.
pub(crate) fn select<'s>(
    source: &'s str,
    input: &'s Input,
    config: &Config,
    grammar: MacroContext,
    provenance: &dyn Inputs,
    charge: &Charge<'_>,
) -> Result<Selected<'s>, LexError> {
    charge(input.tokens.len(), 0)?;
    let bytes = input.tokens.iter().try_fold(0_usize, |sum, token| {
        token
            .end
            .checked_sub(token.start)
            .map(|width| sum.saturating_add(width as usize))
            .ok_or_else(|| error(token.start, "invalid cfg_select input span"))
    })?;
    // Reserve the ordinary parser's temporary node/tree work before it starts;
    // large opaque payload copies are separately charged by the actual input view.
    let token_bound =
        size_of::<crate::syntax::ast::Node>() + size_of::<crate::syntax::token_tree::Tree>() + 64;
    charge(
        0,
        bytes
            .saturating_add(input.tokens.len().saturating_mul(token_bound))
            .saturating_mul(4),
    )?;
    let inputs = Counted {
        inputs: provenance,
        charge,
    };
    let mut parsed = gather(source, input, config, grammar, &inputs, charge)?;
    let chosen = parsed.first.or(parsed.wildcard).ok_or_else(|| {
        error(
            input.tokens.first().map_or(0, |token| token.start),
            "cfg_select has no matching branch",
        )
    })?;
    let mut selected = None;
    let mut calls = parsed.framing_calls;
    // rustc's MacResult checks all other branches before producing the selected one.
    charge(parsed.branches.len(), 0)?;
    for index in (0..parsed.branches.len())
        .filter(|index| *index != chosen)
        .chain(std::iter::once(chosen))
    {
        let row = &parsed.branches[index];
        let ast = if let Some(framed) = row.framed {
            parsed
                .expressions
                .get_mut(framed)
                .and_then(Option::take)
                .ok_or_else(|| error(row.span.start, "missing framed cfg_select expression"))?
        } else {
            charge(row.body.len(), 0)?;
            calls = calls.saturating_add(1);
            fragment::parse_expansion_with_inputs(
                source,
                &input.tokens[row.body.clone()],
                grammar,
                &inputs,
            )?
        };
        if index == chosen {
            selected = Some(ast);
        }
    }
    let ast = selected.ok_or_else(|| error(0, "selected cfg_select syntax was not checked"))?;
    Ok(Selected {
        ast,
        span: parsed.branches[chosen].span,
        parser_calls: calls,
        cfg_uses: parsed.cfg_uses,
    })
}
fn gather<'s>(
    source: &'s str,
    input: &'s Input,
    config: &Config,
    grammar: MacroContext,
    inputs: &dyn Inputs,
    charge: &Charge<'_>,
) -> Result<Gathered<'s>, LexError> {
    let mut result = Gathered::default();
    let mut cursor = 0;
    while cursor < input.tokens.len() {
        let arrow = arrow(input, cursor)?;
        let (wildcard, answer) = predicate(source, input, cursor..arrow, config, inputs, charge)?;
        charge(
            answer.uses.len(),
            answer
                .uses
                .len()
                .saturating_mul(size_of::<config::CfgUse>())
                .saturating_mul(4),
        )?;
        result
            .cfg_uses
            .try_reserve(answer.uses.len())
            .map_err(|_| error(0, "cfg_select gate allocation failed"))?;
        result.cfg_uses.extend(answer.uses);
        if wildcard && result.wildcard.is_none() {
            result.wildcard = Some(result.branches.len());
        }
        if answer.value && result.first.is_none() && result.wildcard.is_none() {
            result.first = Some(result.branches.len());
        }
        let (row, next) = branch(
            source,
            input,
            arrow + 1,
            grammar,
            inputs,
            charge,
            &mut result,
        )?;
        charge(1, size_of::<Branch>().saturating_mul(4))?;
        result
            .branches
            .try_reserve(1)
            .map_err(|_| error(row.span.start, "cfg_select branch allocation failed"))?;
        result.branches.push(row);
        cursor = next;
    }
    Ok(result)
}
fn arrow(input: &Input, mut cursor: usize) -> Result<usize, LexError> {
    let first = cursor;
    if input
        .tokens
        .get(cursor)
        .is_some_and(|token| matches!(token.kind, TokenKind::Punctuation(P::Pound)))
    {
        return Err(error(
            input.tokens[cursor].start,
            "attributes are not allowed on cfg_select branches",
        ));
    }
    while cursor < input.tokens.len() {
        if input.tokens[cursor].kind == TokenKind::Punctuation(P::FatArrow) {
            return Ok(cursor);
        }
        let next = input
            .after
            .get(cursor)
            .copied()
            .ok_or_else(|| error(0, "missing cfg_select delimiter index"))?;
        if next <= cursor || next > input.tokens.len() {
            return Err(error(
                input.tokens[cursor].start,
                "invalid cfg_select delimiter index",
            ));
        }
        cursor = next;
    }
    Err(error(
        input.tokens.get(first).map_or(0, |token| token.start),
        "cfg_select branch requires =>",
    ))
}
fn predicate(
    source: &str,
    input: &Input,
    range: Range<usize>,
    config: &Config,
    inputs: &dyn Inputs,
    charge: &Charge<'_>,
) -> Result<(bool, config::Evaluation), LexError> {
    let offset = input.tokens.get(range.start).map_or(0, |token| token.start);
    if range.is_empty() {
        return Err(error(offset, "cfg_select branch requires a predicate"));
    }
    charge(
        range.len(),
        range
            .len()
            .saturating_mul(size_of::<crate::syntax::token_tree::Tree>())
            .saturating_mul(4),
    )?;
    let mut forest = ForestBuilder::default();
    for &token in &input.tokens[range.clone()] {
        forest.push(source, token)?;
    }
    let forest = forest.finish(input.tokens[range.end - 1].end)?;
    if range.len() == 1
        && input.tokens[range.start].kind == TokenKind::Identifier
        && forest.name(source, forest.roots().start) == Some("_")
    {
        return Ok((
            true,
            config::Evaluation {
                value: false,
                uses: Vec::new(),
            },
        ));
    }
    config::predicate_checked(&forest, source, forest.roots(), config, inputs)
        .map(|answer| (false, answer))
}
fn branch<'s>(
    source: &'s str,
    input: &'s Input,
    start: usize,
    grammar: MacroContext,
    inputs: &dyn Inputs,
    charge: &Charge<'_>,
    result: &mut Gathered<'s>,
) -> Result<(Branch, usize), LexError> {
    let first = input
        .tokens
        .get(start)
        .ok_or_else(|| error(0, "cfg_select branch requires a body"))?;
    if first.kind == TokenKind::Punctuation(P::OpenBrace) {
        let after = *input
            .after
            .get(start)
            .ok_or_else(|| error(first.start, "missing cfg_select body delimiter"))?;
        if after < start + 2
            || after > input.tokens.len()
            || input.tokens[after - 1].kind != TokenKind::Punctuation(P::CloseBrace)
        {
            return Err(error(first.start, "invalid cfg_select body delimiter"));
        }
        let next = after
            + usize::from(
                input
                    .tokens
                    .get(after)
                    .is_some_and(|token| token.kind == TokenKind::Punctuation(P::Comma)),
            );
        return Ok((
            Branch {
                body: start + 1..after - 1,
                span: Span {
                    start: first.end,
                    end: input.tokens[after - 1].start,
                },
                framed: None,
            },
            next,
        ));
    }
    charge(1, 0)?;
    result.framing_calls = result.framing_calls.saturating_add(1);
    let (parsed, complete) = fragment::statement_expression(
        source,
        &input.tokens,
        TokenPosition {
            index: start,
            offset: 0,
        },
        inputs,
    )?;
    if parsed.position.offset != 0 || parsed.position.index <= start {
        return Err(error(first.start, "invalid cfg_select expression boundary"));
    }
    let end = parsed.position.index;
    let comma = input
        .tokens
        .get(end)
        .is_some_and(|token| token.kind == TokenKind::Punctuation(P::Comma));
    if !comma && end < input.tokens.len() && !complete {
        return Err(error(
            input.tokens[end].start,
            "cfg_select expression branch requires a comma",
        ));
    }
    let framed = if grammar == MacroContext::Expression {
        charge(1, size_of::<Option<Ast<'_>>>().saturating_mul(4))?;
        result
            .expressions
            .try_reserve(1)
            .map_err(|_| error(first.start, "cfg_select expression allocation failed"))?;
        let index = result.expressions.len();
        result.expressions.push(Some(parsed.ast));
        Some(index)
    } else {
        None
    };
    Ok((
        Branch {
            body: start..end,
            span: Span {
                start: first.start,
                end: input.tokens[end - 1].end,
            },
            framed,
        },
        end + usize::from(comma),
    ))
}

#[cfg(test)]
#[path = "../../../tests/invariants/chosen_parsing.rs"]
mod tests;
