//! One retained-attribute walk produces requirements and every contributing owner site.
use super::{Contract, FeatureSet, Site, Sources, pay};
use crate::{
    lex::token::Punctuation,
    literal::{self, Value},
    resolve::{
        expansion::Report,
        package::Error,
        unit::{FileId, NodeKey},
    },
    sema::{header_attributes::SourceImage, model::Declarations},
    support::{decorations::Consumer, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::{ItemKind, Safety},
        token_tree::{Delimiter, Forest, TreeKind},
    },
};
pub(in crate::sema) fn collect(
    report: &Report,
    facts: &mut Declarations,
) -> Result<Sources, Error> {
    let mut rows = Sources::default();
    for (index, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(index).map_err(|_| Error::unlocated("too many target-feature files"))?,
        );
        for owner in &unit.configured.owners {
            if !owner.enabled || owner.attributes.is_empty() {
                continue;
            }
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            let source = NodeKey { file, node };
            for (effect, attribute, forest) in
                unit.retained_indexed_attributes(owner.attributes.clone())
            {
                if attribute
                    .name(forest, unit.source.text())
                    .and_then(Consumer::named)
                    != Some(Consumer::TargetFeature)
                {
                    continue;
                }
                let error = |message| Error::at(file, attribute.span.start, message);
                let Kind::Item(index) = unit.configured.ast.node(node).kind else {
                    return Err(error("target_feature requires a function"));
                };
                let ItemKind::Function { flags, .. } =
                    unit.configured.ast.items[index as usize].kind
                else {
                    return Err(error("target_feature requires a function"));
                };
                if !rows.contains_key(&source) {
                    let growth = Growth::map::<NodeKey, Contract>(rows.len(), rows.capacity(), 1);
                    pay(facts, 1 + growth.work, growth.bytes)?;
                    rows.try_reserve(1)
                        .map_err(|_| error("target-feature source allocation failed"))?;
                    rows.insert(
                        source,
                        Contract {
                            source,
                            image: SourceImage::capture(unit, node),
                            sites: Vec::new(),
                            features: FeatureSet::default(),
                            declared_unsafe: flags.safety == Safety::Unsafe,
                        },
                    );
                }
                let row = rows
                    .get_mut(&source)
                    .ok_or_else(|| error("target-feature source disappeared"))?;
                parse(report, facts, source, forest, attribute, &mut row.features)?;
                let growth = Growth::vector::<Site>(row.sites.len(), row.sites.capacity(), 1);
                pay(facts, 1 + growth.work, growth.bytes)?;
                row.sites
                    .try_reserve(1)
                    .map_err(|_| error("target-feature site allocation failed"))?;
                let input = capture_input(facts, forest, attribute).map_err(error)?;
                row.sites.push(Site {
                    owner: NodeKey {
                        file,
                        node: owner.node,
                    },
                    effect,
                    capture: unit.configured.effects[effect].capture,
                    attribute,
                    input,
                });
            }
        }
    }
    Ok(rows)
}
fn parse(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    forest: &Forest,
    attribute: Attribute,
    features: &mut FeatureSet,
) -> Result<(), Error> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Input::Delimited(group) = attribute.input else {
        return Err(error(
            "target_feature requires parenthesized enable arguments",
        ));
    };
    if attribute.unsafe_wrapper
        || !matches!(
            forest.tree(group).map(|tree| tree.kind),
            Some(TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            })
        )
    {
        return Err(error("invalid target_feature attribute form"));
    }
    let range = forest
        .children(group)
        .ok_or_else(|| error("target_feature has no argument group"))?;
    let source = report.package.unit(key.file).source.text();
    let mut cursor = range.start;
    if cursor == range.end {
        return Err(error("target_feature requires enable arguments"));
    }
    while cursor < range.end {
        if forest.name(source, cursor) != Some("enable")
            || !forest.punctuation(cursor + 1, Punctuation::Equal)
        {
            return Err(error("target_feature requires enable = string literal"));
        }
        let token = forest
            .token(cursor + 2)
            .ok_or_else(|| error("target_feature requires a string literal"))?;
        let length = (token.end - token.start) as usize;
        pay(
            facts,
            length.saturating_mul(2),
            Growth::vector::<u8>(0, 0, length).bytes,
        )?;
        let Some(Value::String(bytes)) =
            literal::interpret(source, token).map_err(|e| error(e.message))?
        else {
            return Err(error("target_feature requires a string literal"));
        };
        let names = std::str::from_utf8(bytes.as_ref())
            .map_err(|_| error("target features must be UTF-8"))?;
        let entry = forest
            .tree(cursor)
            .ok_or_else(|| error("target_feature lost its enable entry"))?
            .span;
        let feature_error = |message| Error::at(key.file, entry.start, message);
        for name in names.split(',') {
            features.insert(
                name,
                |growth| pay(facts, growth.work, growth.bytes),
                |gate| {
                    report
                        .package
                        .requested_cpu_gate(gate)
                        .map_err(feature_error)
                },
                feature_error,
            )?;
        }
        cursor += 3;
        if cursor < range.end {
            if !forest.punctuation(cursor, Punctuation::Comma) {
                return Err(error("expected comma between target_feature arguments"));
            }
            cursor += 1;
        }
    }
    Ok(())
}

fn capture_input(
    facts: &mut Declarations,
    forest: &Forest,
    attribute: Attribute,
) -> Result<Vec<(u32, crate::syntax::token_tree::Tree)>, &'static str> {
    let Input::Delimited(group) = attribute.input else {
        return Err("target_feature has no retained input");
    };
    let end = forest.after(group);
    let count = end
        .checked_sub(group)
        .ok_or("invalid target_feature input range")? as usize;
    let growth = Growth::vector::<(u32, crate::syntax::token_tree::Tree)>(0, 0, count);
    pay(facts, count.saturating_add(growth.work), growth.bytes)
        .map_err(|_| "target-feature witness exceeds header budget")?;
    let mut nodes = Vec::new();
    nodes
        .try_reserve_exact(count)
        .map_err(|_| "target-feature witness allocation failed")?;
    for index in group..end {
        nodes.push((
            index,
            forest
                .tree(index)
                .ok_or("target-feature witness lost its tree")?,
        ));
    }
    Ok(nodes)
}
