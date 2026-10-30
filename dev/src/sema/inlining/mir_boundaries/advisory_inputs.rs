//! Parse advisory payloads once without making aliases or suspension permissions.
use super::{
    Attribute, Declarations, Error, Forest, NodeKey, Report, Restriction, reserve_source_contracts,
};
use crate::{
    lex::token::Punctuation,
    syntax::{
        attribute::Input,
        token_tree::{Delimiter, TreeKind},
    },
};
fn string(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    forest: &Forest,
    index: u32,
) -> Result<(), Error> {
    let token = forest
        .token(index)
        .ok_or_else(|| Error::at(key.file, 0, "advisory requires a string literal"))?;
    let bytes = (token.end - token.start) as usize;
    reserve_source_contracts(facts, 1usize.saturating_add(bytes), bytes.saturating_mul(2))?;
    let error = |message: &str| Error::at(key.file, token.start, message);
    let Some(crate::literal::Value::String(value)) =
        crate::literal::interpret(report.package.unit(key.file).source.text(), token)
            .map_err(|e| error(e.message))?
    else {
        return Err(error("advisory requires a string literal"));
    };
    std::str::from_utf8(&value).map_err(|_| error("advisory requires a UTF-8 string literal"))?;
    Ok(())
}
pub(super) fn validate(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    restriction: Restriction,
) -> Result<(), Error> {
    let error = |message: &str| Error::at(key.file, attribute.span.start, message);
    if attribute.unsafe_wrapper {
        return Err(error(&format!(
            "{} cannot use an unsafe wrapper",
            restriction.name()
        )));
    }
    match (restriction, attribute.input) {
        (
            Restriction::Suspend
            | Restriction::MirInline
            | Restriction::Writable
            | Restriction::ConstItemCalls,
            Input::Empty,
        ) => Ok(()),
        (Restriction::Suspend, Input::Equals(range))
            if range.end.checked_sub(range.start) == Some(1) =>
        {
            string(report, facts, key, forest, range.start)
        }
        (Restriction::Confusables, Input::Delimited(group)) => {
            if !forest.tree(group).is_some_and(|tree| {
                matches!(
                    tree.kind,
                    TreeKind::Group {
                        delimiter: Delimiter::Parenthesis,
                        ..
                    }
                )
            }) {
                return Err(error(
                    "rustc_confusables requires a parenthesized string list",
                ));
            }
            let range = forest
                .children(group)
                .ok_or_else(|| error("confusables list disappeared"))?;
            if range.start == range.end {
                return Err(error("rustc_confusables requires at least one string"));
            }
            let mut index = range.start;
            while index < range.end {
                string(report, facts, key, forest, index)?;
                index += 1;
                if index < range.end {
                    if !forest.punctuation(index, Punctuation::Comma) {
                        return Err(error("confusables strings require commas"));
                    }
                    index += 1;
                }
            }
            Ok(())
        }
        (Restriction::Suspend, _) => Err(error(
            "must_not_suspend requires a word or string name-value reason",
        )),
        (Restriction::Confusables, _) => {
            Err(error("rustc_confusables requires a nonempty string list"))
        }
        _ => Err(error(&format!(
            "{} requires a plain word attribute",
            restriction.name()
        ))),
    }
}
