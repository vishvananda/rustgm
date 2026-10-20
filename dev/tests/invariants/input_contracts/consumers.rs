//! Family input contracts reach production declaration and body lifetime consumers.
use super::{Checker, TestResult, report};
use crate::resolve::model::{DefId, DefKind};

const CASES: &[(&str, &str, bool)] = &[
    (
        "nominal-family-region",
        "trait Q{type Out<'a> where Self:'a;}struct Hold<'a,T:Q>(T::Out<'a>);",
        false,
    ),
    (
        "nominal-family-static",
        "trait Q{type Out<T:'static>;}struct Hold<U:Q,T>(U::Out<T>);",
        false,
    ),
    (
        "nominal-family-region-explicit",
        "trait Q{type Out<'a> where Self:'a;}struct Hold<'a,T:Q+'a>(T::Out<'a>);",
        true,
    ),
    (
        "static-default-body",
        "fn needs<T:'static>(){}trait Q{type Out<T:'static>;fn f<T>(x:Self::Out<T>){needs::<T>();}}",
        true,
    ),
    (
        "dependent-default-body",
        "fn needs<T:'static>(){}trait Q{type Out<T> where for<'a>T:'a;fn f<T>(x:Self::Out<T>){needs::<T>();}}",
        false,
    ),
    (
        "erased-nominal-region",
        "trait Q<'a,'b>{type Out where 'a:'b;}impl<'a,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;}struct Hold<'a,'b>(<u8 as Q<'a,'b>>::Out,&'a u8,&'b u8);",
        false,
    ),
    (
        "erased-nominal-region-explicit",
        "trait Q<'a,'b>{type Out where 'a:'b;}impl<'a,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;}struct Hold<'a:'b,'b>(<u8 as Q<'a,'b>>::Out,&'a u8,&'b u8);",
        true,
    ),
    (
        "result-promise-is-not-input",
        "fn needs<T:'static>(){}trait Q{type Out<T>:'static;fn f<T>(x:Self::Out<T>){needs::<T>();}}",
        false,
    ),
];

fn check(source: &str) -> TestResult {
    let report = report(source)?;
    let mut checker = Checker::new(&report)?;
    // Only declaration preparation crosses the deliberately closed public GAT
    // gate. All consumers below are production type and region proof paths.
    super::conformance::prepare(&mut checker)?;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if matches!(
            definition.kind,
            DefKind::Function | DefKind::Const | DefKind::Static
        ) {
            checker.definition(DefId(u32::try_from(index)?), false)?;
            checker.drain(&mut 0, 100_000)?;
        }
    }
    checker.complete_deferred_declarations(&mut 0, 100_000)?;
    checker.validate_associated_type_bounds()?;
    checker.validate_signatures()?;
    checker.validate_trait_implementations()?;
    checker.validate_trait_regions()?;
    checker.local_annotations(&mut 0, 100_000)?;
    checker.complete_alias_normalizations(&mut 0, 100_000)?;
    checker.validate_projection_regions()?;
    let mut facts = crate::sema::bodies::analyze(&report, checker.facts)?;
    let mut templates = crate::body::templates::build(&report, &facts)?;
    crate::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let inputs = crate::body::regions::collect(&facts)?.bodies;
    for input in &inputs {
        input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    let contracts = crate::body::regions::Contracts {
        bodies: inputs,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    crate::borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
    )
    .map_err(|e| e.message)?;
    Ok(())
}

#[test]
fn shared_family_inputs_reach_nominal_declarations_and_default_method_bodies() {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let result = check(source);
        println!("{name}: {result:?}");
        if result.is_ok() != accepted {
            failures.push(format!(
                "{name}: expected accepted={accepted}, result={result:?}"
            ));
        } else if let Err(error) = result {
            let message = error.to_string();
            if ![
                "trait argument lifetime requirement is not satisfied",
                "required lifetime",
                "associated family lifetime contract",
                "associated type lifetime requirement is not satisfied",
            ]
            .iter()
            .any(|class| message.contains(class))
            {
                failures.push(format!("{name}: unexpected diagnostic: {message}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
