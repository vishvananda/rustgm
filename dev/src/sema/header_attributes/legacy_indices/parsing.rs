//! Attribute indexes are parsed tokens; no source call or argument is rewritten.
use super::{Declarations, Error, NodeKey, Report};
use crate::{
    lex::token::Punctuation,
    literal::{self, Value},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input},
        item::{GenericFlavor, ItemKind},
        token_tree::{Delimiter, Forest, TreeKind},
    },
};

pub(super) fn indices(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<Vec<usize>, Error> {
    let error = |message: &str| Error::at(key.file, attribute.span.start, message);
    let Input::Delimited(group) = attribute.input else {
        return Err(error("rustc_legacy_const_generics requires an index list"));
    };
    if attribute.unsafe_wrapper
        || !forest.tree(group).is_some_and(|tree| {
            matches!(
                tree.kind,
                TreeKind::Group {
                    delimiter: Delimiter::Parenthesis,
                    ..
                }
            )
        })
    {
        return Err(error(
            "rustc_legacy_const_generics requires a plain parenthesized list",
        ));
    }
    let range = forest
        .children(group)
        .ok_or_else(|| error("legacy index list disappeared"))?;
    let mut index = range.start;
    let mut result = Vec::new();
    while index < range.end {
        super::super::reserve_source_contracts(facts, 1, 0)?;
        let token = forest
            .token(index)
            .ok_or_else(|| error("legacy index requires an unsuffixed integer literal"))?;
        let Some(Value::Integer(value)) =
            literal::interpret(report.package.unit(key.file).source.text(), token)
                .map_err(|e| error(e.message))?
        else {
            return Err(error("legacy index requires an unsuffixed integer literal"));
        };
        if value.suffix.is_some() {
            return Err(error("legacy index must be unsuffixed"));
        }
        // The pinned parser converts u128 meta integers to host usize. This
        // compiler's common profile is Linux x86-64; retain its exact low64bits.
        let value = u64::try_from(value.magnitude & u128::from(u64::MAX))
            .map_err(|_| error("legacy index normalization failed"))?;
        let value = usize::try_from(value)
            .map_err(|_| error("legacy indexes require the supported 64-bit host"))?;
        let growth =
            crate::support::retained::Growth::vector::<usize>(result.len(), result.capacity(), 1);
        super::super::reserve_source_contracts(facts, growth.work, growth.bytes)?;
        result
            .try_reserve(1)
            .map_err(|_| error("legacy index allocation failed"))?;
        result.push(value);
        index += 1;
        if index < range.end {
            if !forest.punctuation(index, Punctuation::Comma) {
                return Err(error("expected comma between legacy indexes"));
            }
            index += 1;
        }
    }
    if result.is_empty() {
        return Err(error("legacy const generics requires at least one index"));
    }
    Ok(result)
}

pub(super) fn shape(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
) -> Result<(usize, usize), Error> {
    let unit = report.package.unit(key.file);
    let ast = &unit.configured.ast;
    let Kind::Item(index) = ast.node(key.node).kind else {
        return Err(Error::unlocated(
            "legacy marker requires its actual function item",
        ));
    };
    let item = ast.items[index as usize];
    let ItemKind::Function { parameters, .. } = item.kind else {
        return Err(Error::unlocated("legacy marker requires a free function"));
    };
    super::super::charge_source_contract_query(
        facts,
        ast.list(item.generics)
            .len()
            .saturating_add(ast.list(parameters).len())
            .saturating_add(1),
    )?;
    let mut count = 0;
    for mut node in ast.list(item.generics).iter().copied() {
        if unit.configured.active.get(node.get() as usize) != Some(&true) {
            continue;
        }
        while let Kind::Attributed { value, .. } = ast.node(node).kind {
            super::super::charge_source_contract_query(facts, 1)?;
            node = value;
        }
        let Kind::Generic(index) = ast.node(node).kind else {
            return Err(Error::unlocated("legacy generic declaration changed"));
        };
        if ast.generics[index as usize].flavor != GenericFlavor::Const {
            return Err(Error::unlocated(
                "rustc_legacy_const_generics permits only const generic parameters",
            ));
        }
        count += 1;
    }
    let values = ast
        .list(parameters)
        .iter()
        .filter(|node| unit.configured.active.get(node.get() as usize) == Some(&true))
        .count();
    Ok((count, values))
}
