//! The complete owning attribute pass joins a note to its direct regular stable row.
use super::{Draft, Level, Library, ModuleCompatibility, PolicyKind};
use crate::{
    resolve::{
        package::Error,
        unit::{NodeKey, Unit},
    },
    syntax::{
        attribute::{Attribute, Input},
        token_tree::Forest,
    },
};

pub(in crate::resolve) fn collect(
    library: Option<&Library>,
    draft: &mut Option<Draft>,
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &Forest,
    pending: &mut Option<ModuleCompatibility>,
) -> Result<bool, Error> {
    if attribute.name(forest, unit.source.text()) != Some("rustc_allowed_through_unstable_modules")
    {
        return Ok(false);
    }
    let error = |message| Error::at(key.file, attribute.span.start, message);
    if attribute.unsafe_wrapper || !library.is_some_and(|library| library.staged) {
        return Err(error(
            "module compatibility requires staged_api and a safe attribute",
        ));
    }
    if unit.include_context.is_some() {
        return Err(error(
            "included module compatibility requires its actual policy owner",
        ));
    }
    let work = draft.get_or_insert_default();
    work.charge(1, 0)?;
    let Input::Equals(range) = attribute.input else {
        return Err(error("module compatibility requires a string value"));
    };
    if range.end != range.start + 1 {
        return Err(error("module compatibility requires one string literal"));
    }
    let token = forest
        .token(range.start)
        .ok_or_else(|| error("module compatibility literal is absent"))?;
    work.charge(1, (token.end - token.start) as usize * 2)?;
    let Some(crate::literal::Value::String(bytes)) =
        crate::literal::interpret(unit.source.text(), token)
            .map_err(|e| Error::at(key.file, e.offset, e.message))?
    else {
        return Err(error("module compatibility requires a string literal"));
    };
    if bytes.len() > crate::support::crates::generics::source::MAX_REASON_BYTES {
        return Err(error(
            "module compatibility message exceeds the profile limit",
        ));
    }
    let message = std::str::from_utf8(&bytes)
        .map_err(|_| error("module compatibility message is not UTF-8"))?;
    work.charge(message.len(), message.len())?;
    // The pinned attribute callback overwrites earlier valid notes. A repeated
    // attempt still pays its decoded/copy work; an empty message remains present.
    *pending = Some(ModuleCompatibility {
        message: message.into(),
        span: attribute.span,
    });
    Ok(true)
}

pub(in crate::resolve) fn finish(
    draft: &mut Option<Draft>,
    key: NodeKey,
    pending: Option<ModuleCompatibility>,
) -> Result<(), Error> {
    let Some(note) = pending else { return Ok(()) };
    let error = || {
        Error::at(
            key.file,
            note.span.start,
            "module compatibility requires direct regular stable on the same owner",
        )
    };
    let work = draft.as_mut().ok_or_else(error)?;
    work.charge(1, size_of::<ModuleCompatibility>())?;
    let policy = work
        .attributes
        .get(&key)
        .and_then(|id| work.policies.get_mut(id.get() - 1))
        .ok_or_else(error)?;
    if policy.kind != PolicyKind::Regular || policy.origin != key {
        return Err(error());
    }
    let since = match policy.level {
        Level::Stable(since) | Level::StableWithCompatibility { since, .. } => since,
        Level::Unstable { .. } => return Err(error()),
    };
    policy.level = Level::StableWithCompatibility {
        since,
        note: Box::new(note),
    };
    Ok(())
}
