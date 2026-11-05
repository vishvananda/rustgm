//! Checked source families select one transient async application, never executable code.
use crate::{
    body::{coroutines, instantiate::FutureApplication},
    borrow::templates::Checked,
    sema::model::Declarations,
    types::{Kind, TypeId},
};

/// Complete storage from the one actual checked family before immutable cleanup queries.
/// This supplies no generated constructor, poll/drop entry or native permission.
pub(crate) fn checked_future_storage(
    facts: &mut Declarations,
    checked: &Checked<'_>,
) -> Result<(), String> {
    // A completed batch without a deferred family publishes nothing here.
    // This is not global absence or permission to use an unchecked future.
    let Some(arena) = checked.existing_future_store(&facts.types)? else {
        return Ok(());
    };
    let (count, identities) = checked.async_body_identities();
    facts.charge_caller_inputs(count, 0)?;
    let mut published = false;
    for identity in identities {
        let (template, original, regions) = checked
            .async_body(identity)
            .ok_or("future storage requires its actual checked deferred original")?;
        let owner = coroutines::original_template(facts, template, original)?;
        if regions.key != original.key || regions.source != template.body.source {
            return Err("future storage changed its actual checked source family".into());
        }
        facts.infer_checked_async_future(owner, arena)?;
        facts.close_opaque_results(owner, &regions.hidden_regions)?;
        published = true;
    }
    if !published {
        return Err("checked future storage lost its actual batch owner index".into());
    }
    facts.prepare_opaque_storage().map_err(String::from)
}

/// Open the actual checked deferred body for this full future application.
/// This stage supplies no constructor, poll/drop entry, CFG or native permission.
/// # Errors
/// Rejects foreign/stale families, wrong owners/capture frames and failed input proofs.
pub fn future_application(
    facts: &mut Declarations,
    checked: &Checked<'_>,
    future: TypeId,
) -> Result<FutureApplication, String> {
    let Some(Kind::Future { identity, .. }) = facts.types.kind(future) else {
        return Err("async application requires its complete genuine future type".into());
    };
    let arena = checked.future_store(&facts.types)?;
    let (template, original, regions) = checked
        .async_body(identity)
        .ok_or("async application requires its actual checked deferred original")?;
    let definition = coroutines::original_template(facts, template, original)?;
    if regions.key != original.key || regions.source != template.body.source {
        return Err("async application changed its actual checked region owner".into());
    }
    facts.infer_checked_async_future(definition, arena)?;
    facts.close_opaque_results(definition, &regions.hidden_regions)?;
    if facts.checked_future_definition(future)? != definition {
        return Err("async application belongs to another actual checked function".into());
    }
    if facts.types.contains_parameters(future) != Some(false)
        || facts.types.contains_projections(future) != Some(false)
    {
        return Err("selected async application requires closed normalized captures".into());
    }
    crate::body::instantiate::future_application(facts, template, original, future)
}

/// Authenticate constructor readiness without materializing or running the deferred body.
pub(crate) fn future_constructor(
    facts: &mut Declarations,
    checked: &Checked<'_>,
    instance: crate::types::instance::Instance,
) -> Result<coroutines::Constructed, String> {
    let definition = crate::sema::coroutine_methods::constructor_definition(facts, instance)?
        .ok_or("future constructor demand has no actual async function")?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("future constructor lost its actual function")?;
    let actual = facts
        .authored_async_body(function)?
        .ok_or("future constructor lost its actual source contract")?;
    let arena = checked.future_store(&facts.types)?;
    let (template, original, regions) = checked
        .async_body(actual.identity)
        .ok_or("future construction requires its actual checked deferred body")?;
    if coroutines::original_template(facts, template, original)? != definition
        || regions.key != original.key
        || regions.source != template.body.source
    {
        return Err("future construction changed its actual checked source family".into());
    }
    facts.infer_checked_async_future(definition, arena)?;
    facts.close_opaque_results(definition, &regions.hidden_regions)?;
    let mut fold = crate::types::substitute::Bound::new(Vec::new());
    let formation = facts.authored_future_requirements(actual.future, &mut fold, 0)?;
    if !formation.is_empty() {
        return Err("future constructor requires its actual caller formation witnesses".into());
    }
    facts.prepare_opaque_storage()?;
    coroutines::constructor(facts, instance, actual.future).map_err(String::from)
}

/// A cleanup demand selects the real checked family, never a source destructor Instance.
pub(crate) fn future_drop(
    facts: &mut Declarations,
    checked: &Checked<'_>,
    future: TypeId,
) -> Result<coroutines::Constructed, String> {
    let definition = facts.checked_future_definition(future)?;
    let Some(Kind::Future { identity, .. }) = facts.types.kind(future) else {
        return Err("future Drop requires its actual complete application".into());
    };
    let arena = checked.future_store(&facts.types)?;
    let (template, original, regions) = checked
        .async_body(identity)
        .ok_or("future Drop requires its actual checked deferred source family")?;
    if coroutines::original_template(facts, template, original)? != definition
        || regions.key != original.key
        || regions.source != template.body.source
    {
        return Err("future Drop changed its actual checked source family".into());
    }
    facts.infer_checked_async_future(definition, arena)?;
    facts.close_opaque_results(definition, &regions.hidden_regions)?;
    let mut fold = crate::types::substitute::Bound::new(Vec::new());
    let formation = facts.authored_future_requirements(future, &mut fold, 0)?;
    if !formation.is_empty() {
        return Err("future Drop requires its actual closed formation proof".into());
    }
    facts.prepare_opaque_storage()?;
    coroutines::drop_entry(facts, future).map_err(String::from)
}
