//! Factory admission is exact and never permits an ordinary instance without its template.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture() -> Result<(
    crate::resolve::expansion::Report,
    Declarations,
    Vec<Selected>,
)> {
    let mut report = crate::test_source::resolve(
        "fn main(){let mut a=1u8;<u8 as core::ops::AddAssign>::add_assign(&mut a,2);let mut b=3u16;<u16 as core::ops::AddAssign>::add_assign(&mut b,4);}",
        true,
    )?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let mut selected = facts
        .primitive_methods
        .as_ref()
        .ok_or("registry")?
        .selected
        .values()
        .copied()
        .collect::<Vec<_>>();
    selected.sort_by_key(|selected| selected.instance.ty().0);
    assert_eq!(selected.len(), 2);
    Ok((report, facts, selected))
}
fn originals<'a>(
    facts: &mut Declarations,
    templates: &'a crate::body::templates::Templates,
) -> Result<crate::borrow::templates::Checked<'a>> {
    let contracts = crate::body::regions::Contracts {
        bodies: Vec::new(),
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let mut environment = crate::borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut facts.copies,
    };
    Ok(crate::borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &[],
        |_, _| Err("unexpected factory field"),
    )?)
}
fn prove(
    facts: &mut Declarations,
    checked: &crate::borrow::templates::Checked<'_>,
    body: crate::body::ir::Body,
    input: &crate::body::regions::Contract,
    token: Option<crate::body::primitive_methods::Permit>,
) -> Result<crate::borrow::incremental::Proved> {
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let contract = crate::borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        input,
        &body,
    )?;
    let mut environment = crate::borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut facts.copies,
    };
    Ok(crate::borrow::incremental::Verifier::new(None)?.prove(
        &mut environment,
        checked,
        &contract,
        body,
        None,
        token,
    )?)
}
#[test]
fn primitive_factory_requires_matching_selection_and_preserves_ordinary_template_gate() -> Result {
    let (report, mut facts, selected) = fixture()?;
    let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected[0])?;
    let permit = crate::body::primitive_methods::validate(&mut facts, selected[0], &body, &input)?;
    let (other, other_input) =
        crate::body::primitive_methods::build(&report, &mut facts, selected[1])?;
    let other_permit =
        crate::body::primitive_methods::validate(&mut facts, selected[1], &other, &other_input)?;
    let templates = crate::body::templates::Templates { bodies: Vec::new() };
    let checked = originals(&mut facts, &templates)?;
    let error = prove(&mut facts, &checked, body.clone(), &input, None)
        .err()
        .ok_or("ordinary template bypass")?;
    assert!(error.to_string().contains("no checked original template"));
    let error = prove(
        &mut facts,
        &checked,
        body.clone(),
        &input,
        Some(other_permit),
    )
    .err()
    .ok_or("wrong factory instance")?;
    assert!(error.to_string().contains("different selected instance"));
    prove(&mut facts, &checked, body.clone(), &input, Some(permit))?;
    let mut changed = input.clone();
    changed.universals += 1;
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected[0], &body, &changed).is_err()
    );
    let invalid = Selected {
        instance: selected[0].instance,
        descriptor: Descriptor {
            hook: Hook::SubAssign,
            ..selected[0].descriptor
        },
    };
    assert!(invalid.validate(&mut facts).is_err());
    let main = facts
        .functions
        .iter()
        .find(|f| f.body.is_some())
        .ok_or("main")?;
    let ordinary = Instance::new(
        &facts.types,
        facts.definitions[main.definition.0 as usize].ok_or("main type")?,
    )?;
    assert!(
        Selected {
            instance: ordinary,
            descriptor: selected[0].descriptor
        }
        .validate(&mut facts)
        .is_err()
    );
    Ok(())
}
