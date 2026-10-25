//! Consume configured attribute trees once, retaining exact supported lint identities.
use super::{Change, HashMap, Level, Lint, NodeKey, Work};
use crate::{
    lex::token::Punctuation,
    resolve::{expansion::Report, package::Error, unit::FileId},
    syntax::{attribute::Input, token_tree::Forest},
};

pub(super) fn collect(
    report: &Report,
    work: &mut Work,
) -> Result<HashMap<NodeKey, Vec<Change>>, Error> {
    let mut changes: HashMap<NodeKey, Vec<Change>> = HashMap::default();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::at(FileId(0), 0, "too many safety policy files"))?,
        );
        for owner in &unit.configured.owners {
            // Expansion diagnoses duplicate Test before the removed function's
            // own lint attributes take effect. Its retained policy is inherited.
            if !owner.enabled { continue }
            for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
                let name = attribute.name(forest, unit.source.text());
                if name.and_then(crate::support::decorations::Consumer::named)
                    != Some(crate::support::decorations::Consumer::SafetyLint)
                {
                    continue;
                }
                let level = match name {
                    Some("allow") => Level::Allow,
                    Some("warn") => Level::Warn,
                    Some("deny") => Level::Deny,
                    Some("forbid") => Level::Forbid,
                    _ => continue,
                };
                work.attributes += 1;
                let Input::Delimited(group) = attribute.input else {
                    continue;
                };
                let Some(range) = forest.children(group) else {
                    continue;
                };
                let key = NodeKey {
                    file,
                    node: owner.node,
                };
                let mut index = range.start;
                while index < range.end {
                    let start = index;
                    while index < range.end && !forest.punctuation(index, Punctuation::Comma) {
                        index = forest.after(index);
                        work.entries += 1;
                    }
                    if index == start + 1 {
                        let selected = lints(forest, unit.source.text(), start);
                        work.entries += selected.len().saturating_sub(1);
                        for &lint in selected {
                            changes.entry(key).or_default().push(Change { lint, level, offset: attribute.span.start });
                        }
                    }
                    if index < range.end {
                        index += 1;
                    }
                }
            }
        }
    }
    Ok(changes)
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
        Some("unknown_or_malformed_diagnostic_attributes") => &[Lint::DiagnosticAttributes(0), Lint::DiagnosticAttributes(1), Lint::DiagnosticAttributes(2)],
        _ => &[],
    }
}
