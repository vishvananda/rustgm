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
mod predicates;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfgGate {
    UbChecks,
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
    edition: crate::support::edition::Edition,
    runtime_checks: crate::support::runtime_checks::RuntimeCheckConfig,
    flags: HashSet<String>,
    values: HashMap<String, HashSet<String>>,
}

impl Config {
    /// Select the physical source edition; this does not enable a driver profile.
    #[must_use]
    pub fn in_edition(edition: crate::support::edition::Edition) -> Self {
        Self {
            edition,
            ..Self::default()
        }
    }

    #[must_use]
    pub const fn edition(&self) -> crate::support::edition::Edition {
        self.edition
    }

    #[must_use]
    pub const fn runtime_checks(&self) -> crate::support::runtime_checks::RuntimeCheckConfig {
        self.runtime_checks
    }

    /// Set an explicit compilation input before loading immutable source packages.
    pub fn set_runtime_checks(
        &mut self,
        config: crate::support::runtime_checks::RuntimeCheckConfig,
    ) {
        self.runtime_checks = config;
    }

    #[must_use]
    pub fn contains(&self, name: &str, value: Option<&str>) -> bool {
        let Some(value) = value else {
            return if name == "ub_checks" {
                self.runtime_checks.ub_checks()
            } else {
                name == "unix" || self.flags.contains(name)
            };
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
            .is_some_and(|token| keyword::identifier_in(token.kind, name, self.edition))
        {
            return Err(error(0, "expected cfg option identifier"));
        }
        if name.starts_with("target_")
            || matches!(
                name,
                "unix" | "windows" | "panic" | "debug_assertions" | "ub_checks" | "proc_macro"
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
    predicate_with_inputs(forest, source, range, config, &config.edition())
}

/// Evaluate demanded cfg syntax in its original token edition.
/// # Errors
/// Rejects malformed predicates and missing demanded provenance.
pub fn predicate_with_inputs(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<bool, LexError> {
    predicate_checked(forest, source, range, config, inputs).map(|answer| answer.value)
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

/// Keep demanded feature uses from the same typed predicate walk.
pub(crate) fn predicate_checked(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<Evaluation, LexError> {
    predicates::evaluate(forest, source, range, config, inputs, 0)
}
