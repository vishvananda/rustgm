//! Fixed course target facts plus explicit, repeatable custom cfg options.
use crate::support::hash::{HashMap, HashSet};
use crate::{
    lex::{
        keyword,
        token::{Kind, LexError, Punctuation},
    },
    literal::{self, Value},
    syntax::token_tree::{Delimiter, Forest, TreeKind, TreeRange, error},
};
use std::borrow::Cow;

const TARGET_VALUES: &[(&str, &str)] = &[
    ("panic", "abort"),
    ("target_abi", ""),
    ("target_arch", "x86_64"),
    ("target_endian", "little"),
    ("target_env", "gnu"),
    ("target_family", "unix"),
    ("target_feature", "fxsr"),
    ("target_feature", "sse"),
    ("target_feature", "sse2"),
    ("target_os", "linux"),
    ("target_pointer_width", "64"),
    ("target_vendor", "unknown"),
];

#[derive(Default, Debug)]
pub struct Config {
    flags: HashSet<String>,
    values: HashMap<String, HashSet<String>>,
}

impl Config {
    #[must_use]
    pub fn contains(&self, name: &str, value: Option<&str>) -> bool {
        let Some(value) = value else {
            return name == "unix" || self.flags.contains(name);
        };
        TARGET_VALUES.contains(&(name, value))
            || (matches!(
                name,
                "target_has_atomic" | "target_has_atomic_primitive_alignment"
            ) && matches!(value, "8" | "16" | "32" | "64" | "ptr"))
            || self
                .values
                .get(name)
                .is_some_and(|values| values.contains(value))
    }

    /// Add one --cfg setting. Built-in target/profile facts cannot be replaced.
    ///
    /// # Errors
    /// Rejects non-option syntax, malformed strings and compiler-controlled keys.
    pub fn add(&mut self, setting: &str) -> Result<(), LexError> {
        let forest = Forest::parse(setting)?;
        let range = forest.roots();
        let name = forest
            .name(setting, 0)
            .ok_or_else(|| error(0, "expected cfg option name"))?;
        if !forest
            .token(0)
            .is_some_and(|token| keyword::identifier(token.kind, name))
        {
            return Err(error(0, "expected cfg option identifier"));
        }
        if name.starts_with("target_")
            || matches!(
                name,
                "unix" | "windows" | "panic" | "debug_assertions" | "test" | "proc_macro"
            )
        {
            return Err(error(0, "compiler-controlled cfg option"));
        }
        if range.end == 1 {
            self.flags.insert(name.to_owned());
        } else if range.end == 3 && forest.punctuation(1, Punctuation::Equal) {
            self.values
                .entry(name.to_owned())
                .or_default()
                .insert(string(&forest, setting, 2)?.into_owned());
        } else {
            return Err(error(0, "expected cfg name or name = string"));
        }
        Ok(())
    }
}

pub(super) fn string<'source>(
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

/// Validate every operand, even when its truth value cannot change the result.
/// Depth is explicitly bounded; token tree nesting itself is iterative.
///
/// # Errors
/// Rejects malformed predicates and nesting beyond 128 levels.
pub fn predicate(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
) -> Result<bool, LexError> {
    evaluate(forest, source, range, config, 0)
}

fn evaluate(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
    depth: u32,
) -> Result<bool, LexError> {
    let offset = forest.offset(range.start);
    if depth >= 128 {
        return Err(error(offset, "cfg predicate nesting exceeds 128"));
    }
    if range.start == range.end {
        return Err(error(offset, "expected cfg predicate"));
    }
    let name = forest
        .name(source, range.start)
        .ok_or_else(|| error(offset, "expected cfg predicate name"))?;
    let next = range.start + 1;
    let kind = forest
        .token(range.start)
        .map_or(Kind::UnknownPrefix, |token| token.kind);
    if kind == Kind::Identifier && matches!(name, "true" | "false") {
        if next != range.end {
            return Err(error(offset, "boolean cfg predicate takes no arguments"));
        }
        return Ok(name == "true");
    }
    if !keyword::identifier(kind, name) {
        return Err(error(offset, "expected cfg option identifier"));
    }
    if next == range.end {
        return Ok(config.contains(name, None));
    }
    if forest.punctuation(next, Punctuation::Equal) {
        if next + 2 != range.end {
            return Err(error(offset, "expected one cfg string value"));
        }
        return Ok(config.contains(name, Some(&string(forest, source, next + 1)?)));
    }
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
    while index < children.end {
        let (item, following) = comma_item(
            forest,
            TreeRange {
                start: index,
                end: children.end,
            },
        );
        let value = evaluate(forest, source, item, config, depth + 1)?;
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

pub(super) fn comma_item(forest: &Forest, range: TreeRange) -> (TreeRange, u32) {
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
