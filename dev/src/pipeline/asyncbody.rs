//! Checked source families select one transient async application, never executable code.
use crate::{
    body::{coroutines, instantiate::FutureApplication},
    borrow::templates::Checked,
    sema::model::Declarations,
    types::{Kind, TypeId},
};

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
