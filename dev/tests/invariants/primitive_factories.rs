//! Factory admission is exact and never permits an ordinary instance without its template.
use super::*;
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture() -> Result<(
    crate::resolve::expansion::Report,
    Declarations,
    Vec<Selected>,
)> {
    let Factory {
        report,
        facts,
        selected,
        ..
    } = fixture_for(
        "fn main(){let mut a=1u8;<u8 as core::ops::AddAssign>::add_assign(&mut a,2);let mut b=3u16;<u16 as core::ops::AddAssign>::add_assign(&mut b,4);}",
    )?;
    assert_eq!(selected.len(), 2);
    Ok((report, facts, selected))
}
struct Factory {
    report: crate::resolve::expansion::Report,
    facts: Declarations,
    selected: Vec<Selected>,
    definitions: usize,
    instructions: usize,
}
fn fixture_for(source: &str) -> Result<Factory> {
    let mut report = crate::test_source::resolve(source, true)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    let program = crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
    let mut selected = facts
        .primitive_methods
        .as_ref()
        .ok_or("registry")?
        .selected
        .values()
        .copied()
        .collect::<Vec<_>>();
    selected.sort_by_key(|selected| selected.instance.ty().0);
    let bodies = program
        .symbols
        .iter()
        .filter_map(|symbol| symbol.body.as_ref());
    let definitions = bodies.clone().count();
    let instructions = bodies.map(|body| body.instructions.len()).sum();
    Ok(Factory {
        report,
        facts,
        selected,
        definitions,
        instructions,
    })
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

#[test]
fn value_factory_requires_its_exact_form_result_and_output_member() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "fn unrelated<T:core::ops::Sub>(){}fn main(){let _=<u8 as core::ops::Add>::add(2,5);}",
    )?;
    assert_eq!(selected.len(), 1);
    let selected = selected[0];
    assert_eq!(selected.descriptor.form, Form::ValueBinary);
    let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
    assert_eq!(body.result, TypeId::primitive(Primitive::U8));
    assert_eq!(input.universals, 0);
    let permit = crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
    let templates = crate::body::templates::Templates { bodies: Vec::new() };
    let checked = originals(&mut facts, &templates)?;
    assert!(prove(&mut facts, &checked, body.clone(), &input, None).is_err());
    prove(&mut facts, &checked, body.clone(), &input, Some(permit))?;
    let mut wrong = body;
    wrong.result = TypeId::UNIT;
    assert!(
        crate::body::primitive_methods::validate(&mut facts, selected, &wrong, &input).is_err()
    );
    let forged = Selected {
        descriptor: Descriptor {
            form: Form::Assignment,
            ..selected.descriptor
        },
        ..selected
    };
    assert!(forged.validate(&mut facts).is_err());
    let unrelated = facts
        .associated
        .declaration_members()
        .find_map(|(member, owner)| {
            (facts
                .associated
                .operation(owner)
                .is_some_and(|(hook, _)| hook == Hook::Sub)
                && facts
                    .types
                    .projection_schema(crate::types::ItemId(member.0))
                    .is_some())
            .then_some(member)
        })
        .ok_or("unrelated output")?;
    assert!(
        selected
            .descriptor
            .output(
                &facts,
                crate::types::ItemId(unrelated.0),
                crate::types::GenericArgs::EMPTY
            )
            .is_err()
    );
    Ok(())
}
#[test]
fn repeated_generic_add_callers_share_one_primitive_factory() -> Result {
    for width in [8, 32, 128] {
        let mut source = String::from("fn add<T:core::ops::Add<Output=T>>(a:T,b:T)->T{a+b}");
        for i in 0..width {
            write!(source, "fn f{i}(a:u8)->u8{{add(a,1)}}")?;
        }
        source.push_str("fn main(){");
        for i in 0..width {
            write!(source, "let _=f{i}(2);")?;
        }
        source.push('}');
        let factory = fixture_for(&source)?;
        assert_eq!(factory.selected.len(), 1, "width {width}");
        assert!(factory.definitions <= width + 8);
        eprintln!(
            "primitive Add width={width} factories={} definitions={} instructions={} selections={}",
            factory.selected.len(),
            factory.definitions,
            factory.instructions,
            factory.facts.dispatch_dependencies.work.selections
        );
    }
    Ok(())
}
