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
            operation: Operation::Arithmetic(Hook::SubAssign),
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

#[test]
fn every_owned_binary_family_retains_nonempty_exact_primitive_selection() -> Result {
    let operations = [
        (Hook::Sub, "Sub", "sub", 9, 2),
        (Hook::Mul, "Mul", "mul", 3, 4),
        (Hook::Div, "Div", "div", 9, 2),
        (Hook::Rem, "Rem", "rem", 9, 2),
        (Hook::BitAnd, "BitAnd", "bitand", 6, 3),
        (Hook::BitOr, "BitOr", "bitor", 6, 3),
        (Hook::BitXor, "BitXor", "bitxor", 6, 3),
        (Hook::Shl, "Shl", "shl", 3, 2),
        (Hook::Shr, "Shr", "shr", 12, 2),
    ];
    for (hook, name, method, left, right) in operations {
        let source =
            format!("fn main(){{let _=<u8 as core::ops::{name}>::{method}({left},{right});}}");
        let Factory {
            report,
            mut facts,
            selected,
            ..
        } = fixture_for(&source)?;
        assert_eq!(selected.len(), 1, "{name}");
        let selected = selected[0];
        assert_eq!(selected.descriptor.operation, Operation::Arithmetic(hook));
        assert_eq!(selected.descriptor.form, Form::ValueBinary);
        let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selected)?;
        crate::body::primitive_methods::validate(&mut facts, selected, &body, &input)?;
        let forged = Selected {
            descriptor: Descriptor {
                operation: Operation::Arithmetic(Hook::Add),
                ..selected.descriptor
            },
            ..selected
        };
        assert!(forged.validate(&mut facts).is_err());
    }
    Ok(())
}

#[test]
fn primitive_comparison_factory_preserves_both_borrow_slots_and_exact_member() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "fn main(){let a=3u8;let _=<u8 as PartialEq>::eq(&a,&a);let _=<u8 as PartialEq>::ne(&a,&a);}",
    )?;
    assert_eq!(selected.len(), 2);
    let empty = crate::body::templates::Templates { bodies: Vec::new() };
    let checked = originals(&mut facts, &empty)?;
    let mut operators = Vec::new();
    for selection in selected {
        assert_eq!(selection.descriptor.form(), Form::Comparison);
        let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selection)?;
        let source = selection.validate(&mut facts)?;
        operators.push(source.operator);
        assert_eq!(input.universals, 2);
        let signature = input.signature.ok_or("comparison contract signature")?;
        let Some(Kind::Function {
            parameters,
            bound_regions,
            ..
        }) = facts.types.kind(signature)
        else {
            return Err("comparison signature".into());
        };
        assert_eq!(bound_regions, 2);
        for (slot, &parameter) in facts
            .types
            .list(parameters)
            .ok_or("parameters")?
            .iter()
            .enumerate()
        {
            let Some(Kind::Reference {
                mutable: false,
                region,
                ..
            }) = facts.types.kind(parameter)
            else {
                return Err("comparison shared reference".into());
            };
            assert_eq!(region, RegionId::bound(0, u16::try_from(slot)?)?);
        }
        let permit =
            crate::body::primitive_methods::validate(&mut facts, selection, &body, &input)?;
        prove(&mut facts, &checked, body.clone(), &input, Some(permit))?;
        let marker = Selected {
            descriptor: Descriptor {
                operation: Operation::Comparison(crate::support::comparisons::Comparison::Eq),
                ..selection.descriptor
            },
            ..selection
        };
        assert!(marker.validate(&mut facts).is_err());
        let forwarding = Selected {
            descriptor: Descriptor {
                rhs: Rhs::SharedReference,
                ..selection.descriptor
            },
            ..selection
        };
        assert!(forwarding.validate(&mut facts).is_err());
        let mut collapsed = input;
        collapsed.universals = 1;
        assert!(
            crate::body::primitive_methods::validate(&mut facts, selection, &body, &collapsed)
                .is_err()
        );
    }
    assert!(operators.contains(&Punctuation::EqualEqual));
    assert!(operators.contains(&Punctuation::BangEqual));
    Ok(())
}

#[test]
fn primitive_comparison_infers_only_exact_numeric_domains_among_nominal_candidates() -> Result {
    let Factory {
        mut facts,
        selected,
        ..
    } = fixture_for(
        "struct S(u8);impl PartialEq<S> for u8{fn eq(&self,r:&S)->bool{*self==r.0}}fn main(){let _=7u8.eq(&7);let _=7u8.ne(&8);let _=7u8.eq(&S(7));}",
    )?;
    assert_eq!(
        selected.len(),
        2,
        "only primitive/primitive entries use factories"
    );
    let member = selected[0].validate(&mut facts)?.definition;
    let owner = facts
        .associated
        .trait_owner(member)
        .ok_or("comparison owner")?;
    for domain in [infer::Domain::Integer, infer::Domain::Any] {
        let mut context = Context::default();
        let left = context.known(&facts.types, TypeId::primitive(Primitive::U8))?;
        let right = context.variable(domain)?;
        let goal = Predicate::trait_application(
            left,
            crate::types::ItemId(owner.0),
            vec![crate::types::infer::arguments::Argument::Type(right)],
        );
        let candidate = candidate(&facts, &mut context, &goal)?;
        if domain == infer::Domain::Integer {
            assert!(matches!(candidate, Candidate::Present(_)));
            assert!(matches!(
                context.head(&facts.types, right)?,
                Some(Constructor::Primitive(Primitive::U8))
            ));
        } else {
            assert!(matches!(candidate, Candidate::Deferred));
            assert!(context.head(&facts.types, right)?.is_none());
        }
    }
    let mut context = Context::default();
    let left = context.variable(infer::Domain::Integer)?;
    let right = context.variable(infer::Domain::Integer)?;
    let goal = Predicate::trait_application(
        left,
        crate::types::ItemId(owner.0),
        vec![crate::types::infer::arguments::Argument::Type(right)],
    );
    assert!(matches!(
        candidate(&facts, &mut context, &goal)?,
        Candidate::Deferred
    ));
    assert!(context.head(&facts.types, left)?.is_none());
    assert!(context.head(&facts.types, right)?.is_none());
    Ok(())
}

#[test]
fn primitive_ordering_factories_keep_exact_member_results_and_slots() -> Result {
    let Factory {
        report,
        mut facts,
        selected,
        ..
    } = fixture_for(
        "fn main(){let _=u8::partial_cmp(&1,&2);let _=u8::lt(&1,&2);let _=u8::le(&1,&2);let _=u8::gt(&1,&2);let _=u8::ge(&1,&2);let _=u8::cmp(&1,&2);}",
    )?;
    assert_eq!(selected.len(), 6);
    let empty = crate::body::templates::Templates { bodies: Vec::new() };
    let checked = originals(&mut facts, &empty)?;
    let mut methods = std::collections::HashSet::new();
    for selection in selected {
        let source = selection.validate(&mut facts)?;
        let method = source.comparison.ok_or("ordering member")?;
        methods.insert(method);
        let (body, input) = crate::body::primitive_methods::build(&report, &mut facts, selection)?;
        assert_eq!(input.universals, 2);
        for (slot, local) in body.locals.iter().enumerate() {
            assert!(
                matches!(facts.types.kind(local.ty),Some(Kind::Reference{mutable:false,region,..}) if region==RegionId::bound(0,u16::try_from(slot)?)?)
            );
        }
        let permit =
            crate::body::primitive_methods::validate(&mut facts, selection, &body, &input)?;
        prove(&mut facts, &checked, body.clone(), &input, Some(permit))?;
        let marker = Selected {
            descriptor: Descriptor {
                operation: Operation::Comparison(crate::support::comparisons::Comparison::Eq),
                ..selection.descriptor
            },
            ..selection
        };
        assert!(marker.validate(&mut facts).is_err());
        let mut wrong = body.clone();
        wrong.result = TypeId::UNIT;
        assert!(
            crate::body::primitive_methods::validate(&mut facts, selection, &wrong, &input)
                .is_err()
        );
    }
    assert_eq!(methods.len(), 6);
    Ok(())
}
