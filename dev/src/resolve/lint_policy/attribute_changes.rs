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
    let mut parsed = Parsed::default();
    for effect in effects {
        work.charge(key, 1, 0)?;
        let Some((attribute, forest)) = unit.retained_attributes(effect..effect + 1).next() else {
            continue;
        };
        let level = match attribute.name(forest, unit.source.text()) {
            Some("allow") => Some(Level::Allow),
            Some("warn") => Some(Level::Warn),
            Some("deny") => Some(Level::Deny),
            Some("forbid") => Some(Level::Forbid),
            _ => None,
        };
        let trees = witnesses(forest, attribute, level.is_some(), key, work)?;
        work.charge(key, 1, 4 * size_of::<Witness>())?;
        parsed.witnesses.push(Witness {
            effect,
            capture: unit.configured.effects[effect].capture,
            attribute,
            trees,
        });
        let Some(level) = level else { continue };
        parsed.attributes += 1;
        work.attributes += 1;
        let Input::Delimited(group) = attribute.input else {
            continue;
        };
        let Some(range) = forest.children(group) else {
            continue;
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
                let selected = lints(forest, unit.source.text(), start);
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
    }
    Ok(parsed)
}
fn witnesses(
    forest: &Forest,
    attribute: Attribute,
    lint: bool,
    key: NodeKey,
    work: &mut Work,
) -> Result<Vec<(u32, Tree)>, Violation> {
    let input = if lint {
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
    let mut trees = Vec::new();
    for range in [attribute.path, input] {
        if range.start > range.end || range.end as usize > forest.nodes().len() {
            return Err(Violation {
                key,
                offset: attribute.span.start,
                message: "lint source tree witness has an invalid retained range",
            });
        }
        let count = (range.end - range.start) as usize;
        work.charge(key, count, 4 * count * size_of::<(u32, Tree)>())?;
        trees.extend((range.start..range.end).map(|index| (index, forest.nodes()[index as usize])));
    }
    Ok(trees)
}
fn lints(forest: &Forest, source: &str, index: u32) -> &'static [Lint] {
    match forest.name(source, index) {
        Some("unsafe_op_in_unsafe_fn") => &[Lint::Operation],
        Some("static_mut_refs") => &[Lint::Reference],
        Some("invalid_atomic_ordering") => &[Lint::AtomicOrdering],
        Some("unreachable_patterns") => &[Lint::UnreachablePatterns],
        Some("unused_must_use") => &[Lint::MustUse],
        Some("unused_attributes") => &[Lint::UnusedAttributes],
        Some("duplicate_macro_attributes") => &[Lint::DuplicateMacroAttributes],
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
