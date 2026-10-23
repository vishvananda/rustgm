//! Validate retained attribute syntax once while the binding owner pass visits it.
use super::{Draft, Level, Library, Policy, Since};
use crate::{
    lex::token::Punctuation,
    resolve::{
        package::Error,
        unit::{NodeKey, Unit},
    },
    syntax::{
        attribute::{Attribute, Input},
        token_tree::Forest,
    },
};
use std::num::{NonZeroU32, NonZeroUsize};

pub(in crate::resolve) fn collect(
    library: Option<&Library>,
    draft: &mut Option<Draft>,
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
) -> Result<bool, Error> {
    let name = attribute.name(forest, unit.source.text()).unwrap_or("");
    let extended = matches!(
        name,
        "rustc_const_stable"
            | "rustc_const_unstable"
            | "rustc_default_body_unstable"
            | "rustc_allow_const_fn_unstable"
            | "allow_internal_unstable"
            | "rustc_const_stable_indirect"
    );
    if unit.include_context.is_some() {
        return Ok(extended || matches!(name, "stable" | "unstable"));
    }
    if extended {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "const, default-body and macro library stability contracts are not implemented",
        ));
    }
    if !matches!(name, "stable" | "unstable") {
        return Ok(false);
    }
    if attribute.unsafe_wrapper {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "library stability cannot use an unsafe attribute wrapper",
        ));
    }
    if !library.is_some_and(|library| library.staged) {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "library stability requires staged_api",
        ));
    }
    let draft = draft.get_or_insert_with(Draft::default);
    draft.charge(1, 0)?;
    if draft.attributes.contains_key(&key) {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "duplicate or conflicting library stability",
        ));
    }
    let policy = parse(unit, key, attribute, forest, name == "stable", draft)?;
    let id = NonZeroUsize::new(draft.policies.len() + 1).ok_or_else(|| {
        Error::at(
            key.file,
            attribute.span.start,
            "library policy space exhausted",
        )
    })?;
    // Conservative geometric slot charges, including replacement storage.
    draft.charge(
        1,
        4 * (size_of::<Policy>() + size_of::<(NodeKey, NonZeroUsize)>()),
    )?;
    draft.policies.push(policy);
    draft.attributes.insert(key, id);
    Ok(true)
}

fn parse(
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    stable: bool,
    work: &mut Draft,
) -> Result<Policy, Error> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Arguments {
        feature,
        since,
        issue,
        reason,
    } = arguments(unit, key, attribute, forest, stable, work)?;
    let feature: Box<str> = feature.ok_or_else(|| error("library stability requires feature"))?;
    let mut chars = feature.chars();
    if !chars
        .next()
        .is_some_and(|c| c == '_' || unicode_ident::is_xid_start(c))
        || !chars.all(unicode_ident::is_xid_continue)
    {
        return Err(error("library feature must be an identifier"));
    }
    let level = if stable {
        let since = since.ok_or_else(|| error("stable library policy requires since"))?;
        Level::Stable(version(&since).ok_or_else(|| error("invalid stable library version"))?)
    } else {
        let issue = issue.ok_or_else(|| error("unstable library policy requires issue"))?;
        let issue = if issue.as_ref() == "none" {
            None
        } else {
            Some(
                issue
                    .parse::<NonZeroU32>()
                    .map_err(|_| error("invalid unstable library issue"))?,
            )
        };
        Level::Unstable { issue, reason }
    };
    Ok(Policy {
        feature,
        level,
        origin: key,
    })
}
#[derive(Default)]
struct Arguments {
    feature: Option<Box<str>>,
    since: Option<Box<str>>,
    issue: Option<Box<str>>,
    reason: Option<Box<str>>,
}
fn arguments(
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    stable: bool,
    work: &mut Draft,
) -> Result<Arguments, Error> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Input::Delimited(group) = attribute.input else {
        return Err(error("library stability requires named string arguments"));
    };
    let range = forest
        .children(group)
        .ok_or_else(|| error("invalid library stability arguments"))?;
    let mut cursor = range.start;
    let mut arguments = Arguments::default();
    while cursor < range.end {
        work.charge(1, 0)?;
        let name = forest
            .name(unit.source.text(), cursor)
            .ok_or_else(|| error("invalid library stability argument"))?;
        let slot = match name {
            "feature" => &mut arguments.feature,
            "since" if stable => &mut arguments.since,
            "issue" if !stable => &mut arguments.issue,
            "reason" if !stable => &mut arguments.reason,
            "implied_by" | "old_name" if !stable => {
                return Err(error(
                    "implied and renamed library features are not implemented",
                ));
            }
            _ => return Err(error("unsupported library stability argument")),
        };
        if slot.is_some() {
            return Err(error("duplicate library stability argument"));
        }
        if !forest.punctuation(cursor + 1, Punctuation::Equal) || cursor + 2 >= range.end {
            return Err(error(
                "library stability argument requires a string literal",
            ));
        }
        let token = forest
            .token(cursor + 2)
            .ok_or_else(|| error("missing library stability literal"))?;
        work.charge(1, (token.end - token.start) as usize * 2)?;
        let Some(crate::literal::Value::String(bytes)) =
            crate::literal::interpret(unit.source.text(), token)
                .map_err(|e| Error::at(key.file, e.offset, e.message))?
        else {
            return Err(error(
                "library stability argument requires a string literal",
            ));
        };
        *slot = Some(
            std::str::from_utf8(&bytes)
                .map_err(|_| error("invalid library stability string"))?
                .to_owned()
                .into_boxed_str(),
        );
        cursor += 3;
        if cursor < range.end {
            if !forest.punctuation(cursor, Punctuation::Comma) {
                return Err(error("library stability arguments require commas"));
            }
            cursor += 1;
        }
    }
    Ok(arguments)
}

fn version(text: &str) -> Option<Since> {
    if text == "CURRENT_RUSTC_VERSION" {
        return Some(Since::Current);
    }
    if text.contains('-') {
        return None;
    }
    let mut parts = text.splitn(3, '.');
    Some(Since::Version {
        major: parts.next()?.parse().ok()?,
        minor: parts.next()?.parse().ok()?,
        patch: parts.next().unwrap_or("0").parse().ok()?,
    })
}
