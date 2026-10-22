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
            if !owner.enabled {
                continue;
            }
            for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
                let level = match attribute.name(forest, unit.source.text()) {
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
                    if index == start + 1
                        && let Some(lint) = lint(forest, unit.source.text(), start)
                    {
                        changes.entry(key).or_default().push(Change {
                            lint,
                            level,
                            offset: attribute.span.start,
                        });
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
fn lint(forest: &Forest, source: &str, index: u32) -> Option<Lint> {
    match forest.name(source, index)? {
        "unsafe_op_in_unsafe_fn" => Some(Lint::Operation),
        "static_mut_refs" => Some(Lint::Reference),
        "invalid_atomic_ordering" => Some(Lint::AtomicOrdering),
        "unreachable_patterns" => Some(Lint::UnreachablePatterns),
        _ => None,
    }
}
