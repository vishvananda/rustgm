//! Validate retained syntax once, then bind its contract to real canonical items.
use super::{Contract, Declarations, Error, HashMap, HashSet, NodeKey, Report, WarningCode};
use crate::{
    literal::{self, Value},
    resolve::{
        model::{DefId, Namespace},
        unit::FileId,
    },
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::Forest,
    },
};

pub(in crate::sema) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let mut by_source = HashMap::default();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file).map_err(|_| Error::at(FileId(0), 0, "too many must_use files"))?,
        );
        for owner in &unit.configured.owners {
            let owner_key = NodeKey {file, node: owner.node};
            if !owner.enabled {
                super::removed_owners::collect(report, facts, owner_key)?;
                continue;
            }
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            let key = NodeKey { file, node };
            let supported = matches!(unit.configured.ast.node(node).kind, Kind::Item(index) if matches!(unit.configured.ast.items[index as usize].kind,
                ItemKind::Function { .. } | ItemKind::Struct { .. } | ItemKind::Enum { .. } | ItemKind::Union { .. } | ItemKind::Trait { .. }));
            for effect in owner.attributes.clone() {
                let Some((attribute, forest)) = unit.retained_attributes(effect..effect + 1).next()
                else {
                    continue;
                };
                if attribute.name(forest, unit.source.text()) != Some("must_use") {
                    continue;
                }
                let reason = reason(unit.source.text(), forest, attribute)
                    .map_err(|message| Error::at(file, attribute.span.start, message))?;
                if !supported {
                    facts.must_use_diagnostic(
                        report,
                        owner_key,
                        attribute.span.start,
                        0,
                        WarningCode::UnusedAttributes,
                        "must_use attribute requires a function, nominal type or trait".into(),
                    )?;
                    continue;
                }
                if by_source.contains_key(&key) {
                    facts.must_use_diagnostic(
                        report,
                        owner_key,
                        attribute.span.start,
                        0,
                        WarningCode::UnusedAttributes,
                        "duplicate must_use attribute".into(),
                    )?;
                    continue;
                }
                let contracts = facts.must_use_contracts.get_or_insert_default();
                if contracts.sources.len() >= 1_000_000 {
                    return Err(Error::at(
                        file,
                        attribute.span.start,
                        "must_use contract budget exceeded",
                    ));
                }
                let index = contracts.sources.len();
                contracts.work.attributes += 1;
                contracts.work.retained_contracts += 1;
                contracts.work.retained_reasons += reason.as_ref().map_or(0, |reason| reason.len());
                contracts.sources.push(Contract {
                    owner: owner_key,
                    declaration: key,
                    span: attribute.span,
                    effect,
                    reason,
                });
                by_source.insert(key, index);
            }
        }
    }
    bind_definitions(report, facts, &by_source)?;
    if let Some(imported) = &report.resolution.crates {
        let unknown: HashSet<_> = imported
            .definitions()
            .map(|(_, definition)| definition)
            .collect();
        if !unknown.is_empty() {
            facts.must_use_contracts.get_or_insert_default().unknown = unknown;
        }
    }
    Ok(())
}
fn bind_definitions(
    report: &Report,
    facts: &mut Declarations,
    by_source: &HashMap<NodeKey, usize>,
) -> Result<(), Error> {
    if !by_source.is_empty() {
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            let Some(&contract) = definition.node.as_ref().and_then(|key| by_source.get(key))
            else {
                continue;
            };
            let key = definition
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "must_use declaration disappeared"))?;
            let unit = report.package.unit(key.file);
            let namespace = match unit.configured.ast.node(key.node).kind {
                Kind::Item(item)
                    if matches!(
                        unit.configured.ast.items[item as usize].kind,
                        ItemKind::Function { .. }
                    ) =>
                {
                    Namespace::Value
                }
                _ => Namespace::Type,
            };
            if definition.namespace != namespace {
                continue;
            }
            let definition = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "too many must_use definitions"))?,
            );
            let contracts = facts.must_use_contracts.get_or_insert_default();
            contracts.definitions.insert(definition, contract);
            contracts.work.declarations += 1;
        }
    }
    Ok(())
}

fn reason(
    source: &str,
    forest: &Forest,
    attribute: Attribute,
) -> Result<Option<Box<str>>, &'static str> {
    if attribute.unsafe_wrapper {
        return Err("must_use attribute cannot use an unsafe wrapper");
    }
    match attribute.input {
        Input::Empty => Ok(None),
        Input::Equals(range) if range.end == range.start + 1 => {
            let token = forest
                .token(range.start)
                .ok_or("must_use message requires a literal token")?;
            let Some(Value::String(bytes)) =
                literal::interpret(source, token).map_err(|error| error.message)?
            else {
                return Err("must_use message requires a string literal");
            };
            let reason = std::str::from_utf8(bytes.as_ref())
                .map_err(|_| "must_use message is not valid UTF-8")?;
            Ok(Some(reason.into()))
        }
        _ => Err("must_use attribute requires no arguments or one string literal message"),
    }
}
