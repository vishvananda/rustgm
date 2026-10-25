//! Predicate evaluation retains demanded gate uses alongside its boolean data.
use super::{
    CfgGate, CfgUse, Config, Delimiter, Evaluation, Forest, Kind, LexError, Punctuation, TreeKind,
    TreeRange, comma_item, error, keyword, string,
};
use crate::lex::token::Token;
use crate::syntax::token_tree::Span;

fn leaf(value: bool, name: &str, token: Token) -> Result<Evaluation, LexError> {
    let mut uses = Vec::new();
    if name == "ub_checks" {
        uses.try_reserve_exact(1)
            .map_err(|_| error(token.start, "cfg use allocation failed"))?;
        uses.push(CfgUse {
            span: Span {
                start: token.start,
                end: token.end,
            },
            gate: CfgGate::UbChecks,
        });
    }
    Ok(Evaluation { value, uses })
}
fn append(uses: &mut Vec<CfgUse>, mut incoming: Vec<CfgUse>, offset: u32) -> Result<(), LexError> {
    if uses.len().saturating_add(incoming.len()) > 1_000_000 {
        return Err(error(offset, "cfg feature uses exceed one million"));
    }
    uses.try_reserve(incoming.len())
        .map_err(|_| error(offset, "cfg use allocation failed"))?;
    uses.append(&mut incoming);
    Ok(())
}

pub(super) fn evaluate(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
    depth: u32,
) -> Result<Evaluation, LexError> {
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
        let splice = inputs
            .get(token)?
            .ok_or_else(|| error(offset, "missing opaque cfg predicate"))?;
        let captured = splice.metadata()?;
        // metadata() already relocates the typed forest into this occurrence.
        // Keep those spans and raw origins; mapping again would leave its capture.
        return evaluate(
            &captured.forest,
            source,
            captured.forest.roots(),
            config,
            inputs,
            depth + 1,
        );
    }
    let name = forest
        .name(source, range.start)
        .ok_or_else(|| error(offset, "expected cfg predicate name"))?;
    let next = range.start + 1;
    let token = forest
        .token(range.start)
        .ok_or_else(|| error(offset, "expected cfg option token"))?;
    let kind = token.kind;
    if kind == Kind::Identifier && matches!(name, "true" | "false") {
        if next != range.end {
            return Err(error(offset, "boolean cfg predicate takes no arguments"));
        }
        return Ok(Evaluation {
            value: name == "true",
            uses: Vec::new(),
        });
    }
    let edition = if kind == Kind::Identifier && name == "gen" {
        inputs.edition(token)?
    } else {
        config.edition()
    };
    if !keyword::identifier_in(kind, name, edition) {
        return Err(error(offset, "expected cfg option identifier"));
    }
    if next == range.end {
        return leaf(config.contains(name, None), name, token);
    }
    if forest.punctuation(next, Punctuation::Equal) {
        if next + 2 != range.end {
            return Err(error(offset, "expected one cfg string value"));
        }
        return leaf(
            config.contains(name, Some(&string(forest, source, next + 1)?)),
            name,
            token,
        );
    }
    operation(
        forest,
        source,
        range,
        config,
        inputs,
        depth,
        (name, next, offset),
    )
}

fn operation(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
    depth: u32,
    (name, next, offset): (&str, u32, u32),
) -> Result<Evaluation, LexError> {
    if forest.after(next) != range.end
        || !matches!(
            forest.tree(next).map(|t| t.kind),
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
    let mut uses = Vec::new();
    while index < children.end {
        let (item, following) = comma_item(
            forest,
            TreeRange {
                start: index,
                end: children.end,
            },
        );
        let value = evaluate(forest, source, item, config, inputs, depth + 1)?;
        append(&mut uses, value.uses, offset)?;
        count += 1;
        result = match name {
            "all" => result & value.value,
            "any" => result | value.value,
            _ => !value.value,
        };
        index = following;
    }
    if name == "not" && count != 1 {
        return Err(error(offset, "not requires exactly one cfg predicate"));
    }
    Ok(Evaluation {
        value: result,
        uses,
    })
}

#[cfg(test)]
#[path = "../../../tests/invariants/forwarded_configuration.rs"]
mod tests;
