//! Observe the actual retained input and completed proof; never mint a carrier.
use super::*;

pub(super) fn required(
    report: &Report,
    facts: &Declarations,
    prepared: &mut crate::pipeline::Prepared,
) -> Result {
    let function = source_function(report, facts, "clone_from")?;
    // This is the actual Prepared collector later consumed by inspect_selected,
    // not a parallel contract or proof. Its completed row is reused by finish.
    let input = prepared
        .contracts
        .get_or_insert_with(Default::default)
        .function(facts, &function)?;
    let source = function.body.ok_or("actual Clone provided body")?;
    let expected =
        crate::body::defining_cleanups::definition(facts, function.definition, source.into())?
            .ok_or("actual Clone defining owner")?;
    let state = input
        .builtins
        .as_deref()
        .and_then(|r| r.defining())
        .ok_or("actual Clone Required carrier")?;
    let crate::body::defining_cleanups::State::Required(actual) = state else {
        return Err("Clone original did not retain its Required input before preparation".into());
    };
    assert_eq!(*actual, expected);
    assert_eq!(
        input.key,
        body::ir::BodyKey::Declaration(function.definition)
    );
    assert_eq!(input.source, source.into());
    assert_eq!(input.signature, Some(function.signature));
    Ok(())
}

pub(super) fn completed(
    template: &body::templates::Template,
    input: &body::regions::Contract,
    regions: &borrow::regions::Body,
    checked: &borrow::templates::Checked<'_>,
) -> Result {
    let builtins = input.builtins.as_deref().ok_or("actual Clone builtins")?;
    let environment = builtins
        .environment
        .as_ref()
        .ok_or("actual Clone defining Env")?;
    assert!(std::sync::Arc::ptr_eq(environment, &template.environment));
    let state = builtins.defining().ok_or("actual Clone prepared carrier")?;
    assert!(matches!(
        state,
        crate::body::defining_cleanups::State::Prepared(_)
    ));
    let mut work = crate::body::defining_cleanups::Work::new(1_000_000);
    state.check_environment(&template.environment, &mut work)?;
    state.checked(&template.body, Some(&template.environment), &mut work)?;
    assert!(work.steps > 0);
    assert!(
        template
            .destruction
            .sites
            .iter()
            .any(|site| { matches!(site.trigger, body::drops::Trigger::Store(_)) })
    );
    // Checked is constructed only after the ordinary region + Schedule-based
    // loan/cleanup checks. Schedule itself is not retained by this proof API.
    let index = checked
        .regions()
        .bodies
        .iter()
        .position(|candidate| std::ptr::eq(candidate, regions))
        .ok_or("actual paired Clone region result")?;
    assert_eq!(checked.regions().bodies.len(), checked.loans().len());
    let loans = checked
        .loans()
        .get(index)
        .ok_or("actual paired Clone loan result")?;
    println!(
        "Clone defining Required -> Prepared with exact Env; region+loan index={index} points={} checked-work={}",
        loans.points, work.steps
    );
    Ok(())
}
