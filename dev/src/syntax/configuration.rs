//! Shared cfg token grammar collects gate uses once in source order.
use super::{
    interpolation::Inputs,
    token_tree::{Delimiter, Forest, Span, TreeKind, TreeRange, error},
};
use crate::{
    lex::{
        keyword,
        token::{Kind, LexError, Punctuation, Token},
    },
    literal::{self, Value},
    support::edition::Edition,
};
use std::borrow::Cow;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfgGate {
    UbChecks,
    TargetHasAtomic,
    Sanitize,
    FloatReliability,
}
impl CfgGate {
    pub(crate) fn named(name: &str) -> Option<Self> {
        match name {
            "ub_checks" => Some(Self::UbChecks),
            "target_has_atomic_load_store" => Some(Self::TargetHasAtomic),
            "sanitize" => Some(Self::Sanitize),
            "target_has_reliable_f16"
            | "target_has_reliable_f16_math"
            | "target_has_reliable_f128"
            | "target_has_reliable_f128_math" => Some(Self::FloatReliability),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct CfgUse {
    pub span: crate::syntax::token_tree::Span,
    pub gate: CfgGate,
}
pub(crate) struct Evaluation {
    pub value: bool,
    pub uses: Vec<CfgUse>,
}

type Lookup<'a> = dyn Fn(&str, Option<&str>) -> bool + 'a;
struct Context<'a> {
    source: &'a str,
    edition: Option<Edition>,
    inputs: &'a dyn Inputs,
    lookup: Option<&'a Lookup<'a>>,
}

/// Validate metadata without consulting configuration or selecting active owners.
pub(crate) fn documentation(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    inputs: &dyn Inputs,
) -> Result<Vec<CfgUse>, LexError> {
    evaluate(forest, source, range, None, inputs, None, 0).map(|result| result.uses)
}

pub(crate) fn evaluate(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    edition: Option<Edition>,
    inputs: &dyn Inputs,
    lookup: Option<&Lookup<'_>>,
    depth: u32,
) -> Result<Evaluation, LexError> {
    let context = Context {
        source,
        edition,
        inputs,
        lookup,
    };
    let mut uses = Vec::new();
    let value = evaluate_in(forest, range, &context, depth, &mut uses)?;
    Ok(Evaluation { value, uses })
}

fn leaf(
    context: &Context<'_>,
    uses: &mut Vec<CfgUse>,
    name: &str,
    value: Option<&str>,
    token: Token,
) -> Result<bool, LexError> {
    if let Some(gate) = CfgGate::named(name) {
        if uses.len() == 1_000_000 {
            return Err(error(token.start, "cfg feature uses exceed one million"));
        }
        uses.try_reserve(1)
            .map_err(|_| error(token.start, "cfg use allocation failed"))?;
        uses.push(CfgUse {
            span: Span {
                start: token.start,
                end: token.end,
            },
            gate,
        });
    }
    Ok(context.lookup.is_some_and(|lookup| lookup(name, value)))
}

fn evaluate_in(
    forest: &Forest,
    range: TreeRange,
    context: &Context<'_>,
    depth: u32,
    uses: &mut Vec<CfgUse>,
) -> Result<bool, LexError> {
    let offset = forest.offset(range.start);
    if depth >= 128 {
        return Err(error(offset, "cfg predicate nesting exceeds 128"));
    }
    if range.start == range.end {
        return Err(error(offset, "expected cfg predicate"));
    }
    if let Some(token) = forest.token(range.start)
        && matches!(token.kind, Kind::Interpolated(_))
    {
        if range.end != range.start + 1 {
            return Err(error(offset, "opaque cfg predicate has trailing tokens"));
        }
        let splice = context
            .inputs
            .get(token)?
            .ok_or_else(|| error(offset, "missing opaque cfg predicate"))?;
        let captured = splice.metadata()?;
        return evaluate_in(
            &captured.forest,
            captured.forest.roots(),
            context,
            depth + 1,
            uses,
        );
    }
    let name = forest
        .name(context.source, range.start)
        .ok_or_else(|| error(offset, "expected cfg predicate name"))?;
    let next = range.start + 1;
    let token = forest
        .token(range.start)
        .ok_or_else(|| error(offset, "expected cfg option token"))?;
    if token.kind == Kind::Identifier && matches!(name, "true" | "false") {
        if next != range.end {
            return Err(error(offset, "boolean cfg predicate takes no arguments"));
        }
        return Ok(name == "true");
    }
    let edition = if token.kind == Kind::Identifier && name == "gen" {
        context.inputs.edition(token)?
    } else {
        context
            .edition
            .map_or_else(|| context.inputs.edition(token), Ok)?
    };
    if !keyword::identifier_in(token.kind, name, edition) {
        return Err(error(offset, "expected cfg option identifier"));
    }
    if next == range.end {
        return leaf(context, uses, name, None, token);
    }
    if forest.punctuation(next, Punctuation::Equal) {
        if next + 2 != range.end {
            return Err(error(offset, "expected one cfg string value"));
        }
        let value = string_value(forest, context, next + 1)?;
        return leaf(context, uses, name, Some(&value), token);
    }
    operation(forest, range, context, depth, uses, (name, next, offset))
}

fn operation(
    forest: &Forest,
    range: TreeRange,
    context: &Context<'_>,
    depth: u32,
    uses: &mut Vec<CfgUse>,
    (name, next, offset): (&str, u32, u32),
) -> Result<bool, LexError> {
    if forest.after(next) != range.end
        || !matches!(
            forest.tree(next).map(|tree| tree.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        )
    {
        return Err(error(offset, "expected cfg predicate arguments"));
    }
    if !matches!(name, "all" | "any" | "not") {
        return Err(error(offset, "unknown cfg predicate operator"));
    }
    let children = forest
        .children(next)
        .ok_or_else(|| error(offset, "missing cfg predicate arguments"))?;
    let mut index = children.start;
    let mut count = 0;
    let mut result = name == "all";
    while index < children.end {
        let (item, following) = comma_item(
            forest,
            TreeRange {
                start: index,
                end: children.end,
            },
        );
        let value = evaluate_in(forest, item, context, depth + 1, uses)?;
        count += 1;
        result = match name {
            "all" => result & value,
            "any" => result | value,
            _ => !value,
        };
        index = following;
    }
    if name == "not" && count != 1 {
        return Err(error(offset, "not requires exactly one cfg predicate"));
    }
    Ok(result)
}

fn comma_item(forest: &Forest, range: TreeRange) -> (TreeRange, u32) {
    let mut end = range.start;
    while end < range.end && !forest.punctuation(end, Punctuation::Comma) {
        end = forest.after(end);
    }
    (
        TreeRange {
            start: range.start,
            end,
        },
        if end < range.end { end + 1 } else { end },
    )
}

fn string_value<'a>(
    forest: &Forest,
    context: &Context<'a>,
    index: u32,
) -> Result<Cow<'a, str>, LexError> {
    if let Some(token) = forest.token(index)
        && matches!(token.kind, Kind::Interpolated(_))
    {
        return context
            .inputs
            .get(token)?
            .ok_or_else(|| error(token.start, "missing captured cfg string"))?
            .string_literal();
    }
    string(forest, context.source, index)
}

pub(crate) fn string<'source>(
    forest: &Forest,
    source: &'source str,
    index: u32,
) -> Result<Cow<'source, str>, LexError> {
    let offset = forest.offset(index);
    let token = forest
        .token(index)
        .ok_or_else(|| error(offset, "expected cfg string"))?;
    if !matches!(token.kind, Kind::String | Kind::RawString) {
        return Err(error(offset, "expected cfg string"));
    }
    let Some(Value::String(bytes)) = literal::interpret(source, token)? else {
        return Err(error(offset, "expected cfg string"));
    };
    match bytes {
        Cow::Borrowed(bytes) => std::str::from_utf8(bytes)
            .map(Cow::Borrowed)
            .map_err(|_| error(offset, "invalid cfg string encoding")),
        Cow::Owned(bytes) => String::from_utf8(bytes)
            .map(Cow::Owned)
            .map_err(|_| error(offset, "invalid cfg string encoding")),
    }
}
