//! Validate retained attribute syntax once while the binding owner pass visits it.
use super::{Draft, Level, Library, Policy, PolicyKind, Since};
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
use std::borrow::Cow;
use std::num::{NonZeroU32, NonZeroUsize};

pub(in crate::resolve) fn collect(
    library: Option<&Library>,
    draft: &mut Option<Draft<'_>>,
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
    let extended = matches!(name, "allow_internal_unstable" | "allow_internal_unsafe");
    if unit.include_context.is_some() {
        return Ok(extended
            || name == "rustc_default_body_unstable"
            || name == "rustc_allow_const_fn_unstable"
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
    let kind = match name {
        "stable" | "unstable" => PolicyKind::Regular,
        "rustc_const_stable" | "rustc_const_unstable" => PolicyKind::Const,
        "rustc_default_body_unstable" => PolicyKind::DefaultBody,
        _ => return Ok(false),
    };
    if kind == PolicyKind::DefaultBody {
        super::inheritance::validate_target(unit, key, attribute)?;
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
    if draft.attribute(key, kind).is_some() {
        return Err(Error::at(
            key.file,
            attribute.span.start,
            "duplicate or conflicting library stability",
        ));
    }
    let (policy, relationships, first) = parse(
        unit,
        key,
        attribute,
        forest,
        matches!(name, "stable" | "rustc_const_stable"),
        draft,
    )?;
    publish_policy(
        draft,
        key,
        attribute.span,
        policy,
        relationships,
        kind,
        first,
    )?;
    Ok(true)
}

/// Publish the policy and its sparse feature relationships as one prepaid transaction.
fn publish_policy(
    draft: &mut Draft<'_>,
    key: NodeKey,
    span: crate::syntax::token_tree::Span,
    mut policy: Policy,
    relationships: super::relationships::Pending,
    kind: PolicyKind,
    first: Option<super::PolicyId>,
) -> Result<(), Error> {
    policy.kind = kind;
    let id = NonZeroUsize::new(draft.policies.len() + 1)
        .ok_or_else(|| Error::at(key.file, span.start, "library policy space exhausted"))?;
    // Conservative geometric slot charges, including replacement storage.
    draft.charge(
        1,
        4 * (size_of::<Policy>() + size_of::<((NodeKey, PolicyKind), NonZeroUsize)>()),
    )?;
    draft.prepay_relationships(relationships.len())?;
    draft.reserve_feature(first)?;
    draft.policies.push(policy);
    draft.publish_feature(id, first);
    for (kind, name) in relationships.entries() {
        if let Some(name) = name {
            draft
                .relationships
                .push(super::relationships::Relationship {
                    policy: id,
                    name,
                    kind,
                });
        }
    }
    draft.attributes.insert((key, kind), id);
    Ok(())
}

/// API placement records a declaration, without granting const-call permission.
fn record_comptime(
    library: Option<&Library>,
    draft: &mut Option<Draft<'_>>,
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
    work: &mut Draft<'_>,
) -> Result<
    (
        Policy,
        super::relationships::Pending,
        Option<super::PolicyId>,
    ),
    Error,
> {
    let error = |message| Error::at(key.file, attribute.span.start, message);
    let Arguments {
        feature,
        since,
        issue,
        reason,
        implied_by,
        old_name,
    } = arguments(unit, key, attribute, forest, stable, work)?;
    let feature = feature.ok_or_else(|| error("library stability requires feature"))?;
    let (feature, first) = work.feature(&feature, key.file, attribute.span.start)?;
    work.charge(implied_by.as_deref().map_or(0, str::len), 0)?;
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
        Level::Unstable {
            issue,
            reason: reason.map(|value| value.into_owned().into_boxed_str()),
        }
    };
    Ok((
        Policy {
            kind: super::PolicyKind::Regular,
            feature,
            level,
            origin: key,
        },
        super::relationships::Pending {
            implied_by: implied_by.map(|value| value.into_owned().into_boxed_str()),
            old_name: old_name.map(|value| value.into_owned().into_boxed_str()),
        },
        first,
    ))
}
#[derive(Default)]
struct Arguments<'a> {
    feature: Option<Cow<'a, str>>,
    since: Option<Cow<'a, str>>,
    issue: Option<Cow<'a, str>>,
    reason: Option<Cow<'a, str>>,
    implied_by: Option<Cow<'a, str>>,
    old_name: Option<Cow<'a, str>>,
}
fn arguments<'a>(
    unit: &'a Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    stable: bool,
    work: &mut Draft<'_>,
) -> Result<Arguments<'a>, Error> {
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
            "old_name" if !stable => &mut arguments.old_name,
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
        *slot = Some(match bytes {
            Cow::Borrowed(bytes) => Cow::Borrowed(
                std::str::from_utf8(bytes)
                    .map_err(|_| error("invalid library stability string"))?,
            ),
            Cow::Owned(bytes) => Cow::Owned(
                String::from_utf8(bytes).map_err(|_| error("invalid library stability string"))?,
            ),
        });
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

pub(super) fn identifier(name: &str) -> bool {
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
