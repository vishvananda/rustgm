//! Parse retained lint trees once for one authenticated configured owner.
use super::{
    Change, Level, Lint, Violation,
    producer::{Parsed, Witness, Work},
};
use crate::{
    lex::token::Punctuation,
    resolve::unit::{NodeKey, Unit},
    syntax::{
        attribute::{Attribute, Input},
        token_tree::{Forest, Tree, TreeRange},
    },
};
pub(super) fn owner(
    unit: &Unit,
    key: NodeKey,
    tail: super::producer::Tail,
    effects: std::ops::Range<usize>,
    work: &mut Work,
) -> Result<Parsed, Violation> {
    let (rows, pairs) = counts(unit, key, effects.clone(), work)?;
    let bytes = rows
        .saturating_mul(size_of::<Witness>())
        .saturating_add(pairs.saturating_mul(size_of::<(u32, Tree)>()));
    work.charge(key, effects.len().saturating_add(pairs), bytes)?;
    let mut witnesses = Vec::with_capacity(rows);
    let mut trees = Vec::with_capacity(pairs);
    let mut parsed = Parsed::default();
    for (effect, attribute, forest) in unit.retained_indexed_attributes(effects) {
        let covered = coverage(forest, attribute, unit.source.text(), key)?;
        let effect = u32::try_from(effect)
            .map_err(|_| invalid(key, attribute, "lint effect index exceeds 32 bits"))?;
        witnesses.push(Witness {
            effect,
            capture: unit.configured.effects[effect as usize].capture,
            attribute,
            tree_count: covered.count,
        });
        for range in covered.ranges {
            for index in range.start..range.end {
                trees.push((index, forest.nodes()[index as usize]));
            }
        }
        if attribute.name(forest, unit.source.text()) == Some("deprecated") {
            if parsed.deprecated.is_some() {
                return Err(invalid(key, attribute, "duplicate deprecated attribute"));
            }
            parsed.attributes += 1;
            work.attributes += 1;
            parsed.deprecated = Some(Box::new(deprecation(
                forest,
                attribute,
                unit.source.text(),
                key,
                work,
            )?));
        }
        if let Some(level) = covered.level {
            changes(
                &mut parsed,
                forest,
                attribute,
                (level, tail, effect),
                unit.source.text(),
                key,
                work,
            )?;
        }
    }
    // The pinned Global allocator reports exact requested capacity. Exact push
    // counts leave no shrink/copy allocation when these buffers become boxes.
    if witnesses.len() != rows
        || witnesses.capacity() != rows
        || trees.len() != pairs
        || trees.capacity() != pairs
    {
        return Err(super::producer::invalid(
            key,
            "lint witness allocation changed its exact extent",
        ));
    }
    parsed.witnesses = witnesses.into_boxed_slice();
    parsed.trees = trees.into_boxed_slice();
    Ok(parsed)
}
fn counts(
    unit: &Unit,
    key: NodeKey,
    effects: std::ops::Range<usize>,
    work: &mut Work,
) -> Result<(usize, usize), Violation> {
    work.charge(key, effects.len().saturating_mul(2), 0)?;
    if unit
        .configured
        .effects
        .get(effects.clone())
        .is_none_or(|values| {
            values
                .iter()
                .any(|effect| effect.capture as usize >= unit.configured.ast.attributes.len())
        })
    {
        return Err(super::producer::invalid(
            key,
            "lint source effects or captures are missing",
        ));
    }
    let (mut rows, mut pairs) = (0_usize, 0_usize);
    for (effect, attribute, forest) in unit.retained_indexed_attributes(effects) {
        u32::try_from(effect)
            .map_err(|_| invalid(key, attribute, "lint effect index exceeds 32 bits"))?;
        let covered = coverage(forest, attribute, unit.source.text(), key)?;
        rows = rows.saturating_add(1);
        pairs = pairs.saturating_add(covered.count as usize);
    }
    Ok((rows, pairs))
}
pub(super) struct Coverage {
    pub ranges: [TreeRange; 2],
    pub count: u32,
    level: Option<Level>,
}
pub(super) fn coverage(
    forest: &Forest,
    attribute: Attribute,
    source: &str,
    key: NodeKey,
) -> Result<Coverage, Violation> {
    if attribute.path.start > attribute.path.end
        || attribute.path.end as usize > forest.nodes().len()
    {
        return Err(invalid(
            key,
            attribute,
            "lint source path witness has an invalid retained range",
        ));
    }
    let name = if matches!(attribute.input, Input::Doc(_))
        || attribute.path.end - attribute.path.start == 1
    {
        attribute.name(forest, source)
    } else {
        None
    };
    let level = match name {
        Some("expect") => Some(Level::Expect),
        Some("allow") => Some(Level::Allow),
        Some("warn") => Some(Level::Warn),
        Some("deny") => Some(Level::Deny),
        Some("forbid") => Some(Level::Forbid),
        _ => None,
    };
    let input = if level.is_some()
        || matches!(
            name,
            Some("deprecated" | "unstable" | "unstable_feature_bound")
        ) {
        match attribute.input {
            Input::Delimited(group) => TreeRange {
                start: group,
                end: forest.after(group),
            },
            Input::Equals(range) => range,
            Input::Empty | Input::Doc(_) => TreeRange { start: 0, end: 0 },
        }
    } else {
        TreeRange { start: 0, end: 0 }
    };
    let ranges = [attribute.path, input];
    let mut count = 0_u32;
    for range in ranges {
        if range.start > range.end || range.end as usize > forest.nodes().len() {
            return Err(invalid(
                key,
                attribute,
                "lint source tree witness has an invalid retained range",
            ));
        }
        count = count
            .checked_add(range.end - range.start)
            .ok_or_else(|| invalid(key, attribute, "lint witness tree count exceeds 32 bits"))?;
    }
    Ok(Coverage {
        ranges,
        count,
        level,
    })
}
fn changes(
    parsed: &mut Parsed,
    forest: &Forest,
    attribute: Attribute,
    setting: (Level, super::producer::Tail, u32),
    source: &str,
    key: NodeKey,
    work: &mut Work,
) -> Result<(), Violation> {
    let (level, tail, effect) = setting;
    parsed.attributes += 1;
    work.attributes += 1;
    let Input::Delimited(group) = attribute.input else {
        return Err(invalid(
            key,
            attribute,
            "lint attribute requires a parenthesized lint list",
        ));
    };
    let range = forest
        .children(group)
        .ok_or_else(|| invalid(key, attribute, "invalid lint list"))?;
    let mut index = range.start;
    let mut ordinal = 0_u32;
    let first = parsed.expectations.len();
    let mut reason = None;
    while index < range.end {
        let start = index;
        while index < range.end && !forest.punctuation(index, Punctuation::Comma) {
            work.charge(key, 1, 0)?;
            index = forest.after(index);
            parsed.entries += 1;
            work.entries += 1;
        }
        if forest.name(source, start) == Some("reason") {
            if reason.is_some()
                || index != start + 3
                || !forest.punctuation(start + 1, Punctuation::Equal)
            {
                return Err(invalid(
                    key,
                    attribute,
                    "lint reason requires one string literal",
                ));
            }
            reason = Some(string(forest, source, start + 2, key, attribute, work)?);
            if index < range.end && index + 1 != range.end {
                return Err(invalid(
                    key,
                    attribute,
                    "lint reason must be the last argument",
                ));
            }
        } else {
            let selected = if index == start + 1 {
                lints(forest, source, start)
            } else {
                &[]
            };
            if selected.is_empty() && level == Level::Expect {
                return Err(invalid(
                    key,
                    attribute,
                    "unsupported lint expectation requires an implemented lint producer",
                ));
            }
            let expectation = if level == Level::Expect {
                Some(register(
                    parsed,
                    attribute,
                    (tail, effect, ordinal),
                    selected,
                    key,
                    work,
                )?)
            } else {
                None
            };
            parsed.entries += selected.len().saturating_sub(1);
            work.entries += selected.len().saturating_sub(1);
            for &lint in selected {
                work.charge(key, 1, 4 * size_of::<Change>())?;
                parsed.changes.push(Change {
                    lint,
                    level,
                    offset: attribute.span.start,
                    expectation,
                });
            }
            ordinal = ordinal
                .checked_add(1)
                .ok_or_else(|| invalid(key, attribute, "lint list ordinal overflow"))?;
        }
        if index < range.end {
            index += 1;
        }
    }
    if let Some(reason) = reason {
        for row in &mut parsed.expectations[first..] {
            work.charge(key, 1 + reason.len(), reason.len())?;
            row.reason = Some(reason.clone());
        }
    }
    Ok(())
}
fn register(
    parsed: &mut Parsed,
    attribute: Attribute,
    identity: (super::producer::Tail, u32, u32),
    selected: &[Lint],
    key: NodeKey,
    work: &mut Work,
) -> Result<super::ExpectationId, Violation> {
    let id = super::ExpectationId {
        owner: identity.0,
        index: u32::try_from(parsed.expectations.len())
            .map_err(|_| invalid(key, attribute, "expectation ordinal overflow"))?,
    };
    let growth = crate::support::retained::Growth::vector::<super::expectations::Expectation>(
        parsed.expectations.len(),
        parsed.expectations.capacity(),
        1,
    );
    work.charge(key, 1 + growth.work, growth.bytes)?;
    work.expectations = work.expectations.saturating_add(1);
    parsed.expectations.push(super::expectations::Expectation {
        effect: identity.1,
        ordinal: identity.2,
        span: attribute.span,
        lint: selected[0],
        members: selected
            .iter()
            .try_fold(0_u16, |mask, lint| lint.mask().map(|member| mask | member))
            .ok_or_else(|| invalid(key, attribute, "invalid lint expectation member"))?,
        reason: None,
        early: std::cell::Cell::new(false),
    });
    Ok(id)
}
fn string(
    forest: &Forest,
    source: &str,
    index: u32,
    key: NodeKey,
    attribute: Attribute,
    work: &mut Work,
) -> Result<Box<str>, Violation> {
    let token = forest
        .token(index)
        .ok_or_else(|| invalid(key, attribute, "attribute requires a string literal"))?;
    work.charge(
        key,
        1 + (token.end - token.start) as usize,
        2 * (token.end - token.start) as usize,
    )?;
    let Some(crate::literal::Value::String(bytes)) = crate::literal::interpret(source, token)
        .map_err(|_| invalid(key, attribute, "invalid attribute string literal"))?
    else {
        return Err(invalid(
            key,
            attribute,
            "attribute requires a string literal",
        ));
    };
    String::from_utf8(bytes.into_owned())
        .map(String::into_boxed_str)
        .map_err(|_| invalid(key, attribute, "attribute requires UTF-8 string data"))
}
fn deprecation(
    forest: &Forest,
    attribute: Attribute,
    source: &str,
    key: NodeKey,
    work: &mut Work,
) -> Result<super::expectations::Deprecation, Violation> {
    if attribute.unsafe_wrapper {
        return Err(invalid(
            key,
            attribute,
            "deprecated is not an unsafe attribute",
        ));
    }
    work.charge(key, 1, size_of::<super::expectations::Deprecation>())?;
    let mut result = super::expectations::Deprecation {
        span: attribute.span,
        note: None,
        since: None,
        suggestion: None,
        staged_unsupported: false,
    };
    if attribute.input == Input::Empty {
        return Ok(result);
    }
    let Input::Delimited(group) = attribute.input else {
        return Err(invalid(
            key,
            attribute,
            "deprecated requires named string arguments",
        ));
    };
    let range = forest
        .children(group)
        .ok_or_else(|| invalid(key, attribute, "invalid deprecated arguments"))?;
    let mut index = range.start;
    while index < range.end {
        work.charge(key, 1, 0)?;
        let name = forest.name(source, index);
        let occupied = match name {
            Some("note") => result.note.is_some(),
            Some("since") => result.since.is_some(),
            Some("suggestion") => result.suggestion.is_some(),
            _ => return Err(invalid(key, attribute, "unsupported deprecated argument")),
        };
        if occupied || index + 2 >= range.end || !forest.punctuation(index + 1, Punctuation::Equal)
        {
            return Err(invalid(
                key,
                attribute,
                "deprecated requires unique named string arguments",
            ));
        }
        let value = string(forest, source, index + 2, key, attribute, work)?;
        match name {
            Some("note") => result.note = Some(value),
            Some("since") => result.since = Some(value),
            _ => {
                let span = crate::syntax::token_tree::Span {
                    start: forest
                        .tree(index)
                        .ok_or_else(|| invalid(key, attribute, "missing suggestion name"))?
                        .span
                        .start,
                    end: forest
                        .tree(index + 2)
                        .ok_or_else(|| invalid(key, attribute, "missing suggestion value"))?
                        .span
                        .end,
                };
                result.suggestion = Some((value, span));
            }
        }
        index += 3;
        if index < range.end {
            if !forest.punctuation(index, Punctuation::Comma) {
                return Err(invalid(
                    key,
                    attribute,
                    "deprecated arguments require commas",
                ));
            }
            index += 1;
        }
    }
    result.staged_unsupported = result.since.as_deref().is_some_and(|since| {
        let mut parts = since.split('.');
        let Some(major) = parts.next().and_then(|part| part.parse::<u32>().ok()) else {
            return true;
        };
        let Some(minor) = parts.next().and_then(|part| part.parse::<u32>().ok()) else {
            return true;
        };
        let Some(_patch) = parts.next().and_then(|part| part.parse::<u32>().ok()) else {
            return true;
        };
        parts.next().is_some() || major > 1 || (major == 1 && minor > 98)
    });
    Ok(result)
}
fn invalid(key: NodeKey, attribute: Attribute, message: &'static str) -> Violation {
    Violation {
        key,
        offset: attribute.span.start,
        message,
    }
}
fn lints(forest: &Forest, source: &str, index: u32) -> &'static [Lint] {
    match forest.name(source, index) {
        Some("deprecated") => &[Lint::Deprecated],
        Some("hidden_glob_reexports") => &[Lint::HiddenGlobReexports],
        Some("ineffective_unstable_trait_impl") => &[Lint::IneffectiveUnstableTraitImpl],
        Some("unfulfilled_lint_expectations") => &[Lint::UnfulfilledExpectations],
        Some("unsafe_op_in_unsafe_fn") => &[Lint::Operation],
        Some("static_mut_refs") => &[Lint::Reference],
        Some("invalid_atomic_ordering") => &[Lint::AtomicOrdering],
        Some("unreachable_patterns") => &[Lint::UnreachablePatterns],
        Some("unused_must_use") => &[Lint::MustUse],
        Some("unused_attributes") => &[Lint::UnusedAttributes],
        Some("unused_visibilities") => &[Lint::UnusedVisibilities],
        Some("duplicate_macro_attributes") => &[Lint::DuplicateMacroAttributes],
        Some("ambiguous_derive_helpers" | "future_incompatible") => &[Lint::AmbiguousDeriveHelpers],
        Some("malformed_diagnostic_attributes") => &[Lint::DiagnosticAttributes(0)],
        Some("malformed_diagnostic_format_literals") => &[Lint::DiagnosticAttributes(1)],
        Some("misplaced_diagnostic_attributes") => &[Lint::DiagnosticAttributes(2)],
        Some("unknown_or_malformed_diagnostic_attributes") => &[
            Lint::DiagnosticAttributes(0),
            Lint::DiagnosticAttributes(1),
            Lint::DiagnosticAttributes(2),
        ],
        _ => &[],
    }
}
