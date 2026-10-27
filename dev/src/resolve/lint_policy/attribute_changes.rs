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
        if let Some(level) = covered.level {
            changes(
                &mut parsed,
                forest,
                attribute,
                level,
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
        Some("allow") => Some(Level::Allow),
        Some("warn") => Some(Level::Warn),
        Some("deny") => Some(Level::Deny),
        Some("forbid") => Some(Level::Forbid),
        _ => None,
    };
    let input = if level.is_some() {
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
    level: Level,
    source: &str,
    key: NodeKey,
    work: &mut Work,
) -> Result<(), Violation> {
    parsed.attributes += 1;
    work.attributes += 1;
    let Input::Delimited(group) = attribute.input else {
        return Ok(());
    };
    let Some(range) = forest.children(group) else {
        return Ok(());
    };
    let mut index = range.start;
    while index < range.end {
        let start = index;
        while index < range.end && !forest.punctuation(index, Punctuation::Comma) {
            work.charge(key, 1, 0)?;
            index = forest.after(index);
            parsed.entries += 1;
            work.entries += 1;
        }
        if index == start + 1 {
            let selected = lints(forest, source, start);
            parsed.entries += selected.len().saturating_sub(1);
            work.entries += selected.len().saturating_sub(1);
            for &lint in selected {
                work.charge(key, 1, 4 * size_of::<Change>())?;
                parsed.changes.push(Change {
                    lint,
                    level,
                    offset: attribute.span.start,
                });
            }
        }
        if index < range.end {
            index += 1;
        }
    }
    Ok(())
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
