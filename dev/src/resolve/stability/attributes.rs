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
    allowance: Option<crate::resolve::bindings::panic_contracts::Allowance>,
) -> Result<bool, Error> {
    let name = attribute.name(forest, unit.source.text()).unwrap_or("");
    if record_comptime(library, draft, unit, key, attribute, name)? {
        return Ok(false);
    }
    let extended = matches!(
        name,
        "rustc_default_body_unstable"
            | "rustc_allow_const_fn_unstable"
            | "allow_internal_unstable"
            | "allow_internal_unsafe"
    );
    if unit.include_context.is_some() {
        return Ok(extended
            || matches!(
                name,
                "stable" | "unstable" | "rustc_const_stable" | "rustc_const_unstable"
            ));
    }
    if matches!(name, "allow_internal_unstable" | "allow_internal_unsafe")
        && allowance.is_some_and(|allowance| allowance.matches(key, attribute))
    {
        return Ok(true);
    }
    if extended {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "const, default-body and macro library stability contracts are not implemented",
        ));
    }
    let constant = matches!(name, "rustc_const_stable" | "rustc_const_unstable");
    if !constant && !matches!(name, "stable" | "unstable") {
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
    if (if constant {
        &draft.const_attributes
    } else {
        &draft.attributes
    })
    .contains_key(&key)
    {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "duplicate or conflicting library stability",
        ));
    }
    let (policy, implied_by) = parse(
        unit,
        key,
        attribute,
        forest,
        matches!(name, "stable" | "rustc_const_stable"),
        draft,
    )?;
    publish_policy(draft, key, attribute.span, policy, implied_by, constant)?;
    Ok(true)
}

/// Publish the policy and its optional implication as one prepaid transaction.
fn publish_policy(
    draft: &mut Draft,
    key: NodeKey,
    span: crate::syntax::token_tree::Span,
    mut policy: Policy,
    implied_by: Option<Box<str>>,
    constant: bool,
) -> Result<(), Error> {
    policy.kind = if constant {
        super::PolicyKind::Const
    } else {
        super::PolicyKind::Regular
    };
    let id = NonZeroUsize::new(draft.policies.len() + 1)
        .ok_or_else(|| Error::at(key.file, span.start, "library policy space exhausted"))?;
    // Conservative geometric slot charges, including replacement storage.
    draft.charge(
        1,
        4 * (size_of::<Policy>() + size_of::<(NodeKey, NonZeroUsize)>()),
    )?;
    if implied_by.is_some() {
        draft.prepay_implication()?;
    }
    draft.policies.push(policy);
    if let Some(by) = implied_by {
        draft
            .implications
            .push(super::implications::Implication { policy: id, by });
    }
    if constant {
        draft.const_attributes.insert(key, id);
    } else {
        draft.attributes.insert(key, id);
    }
    Ok(())
}

/// API placement records a declaration, without granting const-call permission.
fn record_comptime(
    library: Option<&Library>,
    draft: &mut Option<Draft>,
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    name: &str,
) -> Result<bool, Error> {
    if !matches!(name, "rustc_comptime" | "rustc_non_const_trait_method")
        || !library.is_some_and(|library| library.staged)
        || unit.include_context.is_some()
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Ok(false);
    }
    draft.get_or_insert_with(Draft::default).record_marker(
        key,
        if name == "rustc_comptime" {
            super::FunctionMarker::Comptime
        } else {
            super::FunctionMarker::NonConstMember
        },
    )?;
    Ok(true)
}

fn parse(
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    stable: bool,
    work: &mut Draft,
) -> Result<(Policy, Option<Box<str>>), Error> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Arguments {
        feature,
        since,
        issue,
        reason,
        implied_by,
    } = arguments(unit, key, attribute, forest, stable, work)?;
    let feature: Box<str> = feature.ok_or_else(|| error("library stability requires feature"))?;
    work.charge(
        feature
            .len()
            .saturating_add(implied_by.as_deref().map_or(0, str::len)),
        0,
    )?;
    if !identifier(&feature) {
        return Err(error("library feature must be an identifier"));
    }
    if implied_by.as_deref().is_some_and(|name| !identifier(name)) {
        return Err(error("implied library feature must be an identifier"));
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
    Ok((
        Policy {
            kind: super::PolicyKind::Regular,
            feature,
            level,
            origin: key,
        },
        implied_by,
    ))
}
#[derive(Default)]
struct Arguments {
    feature: Option<Box<str>>,
    since: Option<Box<str>>,
    issue: Option<Box<str>>,
    reason: Option<Box<str>>,
    implied_by: Option<Box<str>>,
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
            "implied_by" if !stable => &mut arguments.implied_by,
            "old_name" if !stable => {
                return Err(error("renamed library features are not implemented"));
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

fn identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c == '_' || unicode_ident::is_xid_start(c))
        && chars.all(unicode_ident::is_xid_continue)
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
