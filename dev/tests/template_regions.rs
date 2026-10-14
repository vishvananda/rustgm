#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    ("identity", "fn f<T>(x:&T)->&T{x}", true),
    ("shared-reborrow", "fn f<T>(x:&mut T)->&T{&*x}", true),
    (
        "mutable-reborrow",
        "fn f<T>(x:&mut T)->&mut T{&mut*x}",
        true,
    ),
    ("local-return", "fn f<'a,T>(x:T)->&'a T{&x}", false),
    ("temporary-return", "fn f<'a,T>(x:T)->&'a T{&{x}}", false),
    (
        "copy-temporary",
        "fn f<T:Copy+'static>(x:T)->&'static T{&{x}}",
        false,
    ),
    (
        "forward-reference",
        "fn f<'a,T>(x:&'a T)->&'a T{let y=x;&*y}",
        true,
    ),
    (
        "forward-call",
        "fn id<T>(x:&T)->&T{x}fn f<'a,T>(x:T)->&'a T{id(&x)}",
        false,
    ),
    ("literal-promotion", "fn f<T>()->&'static u8{&42u8}", true),
    (
        "constant-promotion",
        "const N:u8=3;fn f<T>()->&'static u8{&N}",
        true,
    ),
    ("empty-promotion", "fn f<T>()->&'static[u8;0]{&[]}", true),
    (
        "generic-empty",
        "fn f<T:'static>()->&'static[T;0]{&[]}",
        true,
    ),
    (
        "generic-mutable-empty",
        "fn f<T:'static>()->&'static mut[T;0]{&mut []}",
        true,
    ),
    (
        "generic-nested-empty",
        "fn f<T:'static>()->&'static &'static[T;0]{&&[]}",
        true,
    ),
    (
        "zero-repeat",
        "fn f<T:'static>(x:T)->&'static[T;0]{&[x;0]}",
        true,
    ),
    (
        "nonzero-repeat",
        "fn f<T:'static>(x:T)->&'static[T;1]{&[x;1]}",
        false,
    ),
    (
        "drop-promotion",
        "struct D;impl Drop for D{fn drop(&mut self){}}fn f<T>()->&'static D{&D}",
        false,
    ),
    (
        "drop-constant",
        "struct D;impl Drop for D{fn drop(&mut self){}}const C:D=D;fn f<T>()->&'static D{&C}",
        false,
    ),
    (
        "drop-empty",
        "struct D;impl Drop for D{fn drop(&mut self){}}fn f<T>()->&'static[D;0]{&[]}",
        true,
    ),
    ("shortening", "fn f<'a:'b,'b,T>(x:&'a T)->&'b T{x}", true),
    ("lengthening", "fn f<'a,'b:'a,T>(x:&'a T)->&'b T{x}", false),
    (
        "array-shortening",
        "fn f<'a:'b,'b,T,const N:usize>(x:[&'a T;N])->[&'b T;N]{x}",
        true,
    ),
    (
        "array-lengthening",
        "fn f<'a,'b:'a,T,const N:usize>(x:[&'a T;N])->[&'b T;N]{x}",
        false,
    ),
    (
        "array-invariant",
        "fn f<'a:'b,'b,T,const N:usize>(x:&'a mut[&'a T;N])->&'b mut[&'b T;N]{x}",
        false,
    ),
    (
        "array-distinct-lengths",
        "fn f<T,const N:usize,const M:usize>(x:&[T;N])->&[T;M]{x}",
        false,
    ),
];

fn prove(
    facts: &rustgm::sema::model::Declarations,
    templates: &rustgm::body::templates::Templates,
) -> Result<rustgm::borrow::regions::Analysis, Box<dyn std::error::Error>> {
    let contracts = rustgm::body::regions::collect(facts)?;
    Ok(rustgm::borrow::regions::check_templates(
        &facts.types,
        &facts.aggregates,
        &facts.lifetimes.variances,
        &contracts,
        templates,
    )?)
}

fn check(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = rustgm::body::templates::build(&report, &facts)?;
    rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    prove(&facts, &templates)?;
    Ok(())
}

#[test]
fn declared_template_provenance_outcomes_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("template-region-{name}"), source, *accepted)?;
    }
    Ok(())
}

#[test]
fn templates_use_real_borrow_origins_and_scoped_promotion() -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        let result = check(source);
        if result.is_ok() != *accepted {
            failures.push(format!("{name}: wanted {accepted}; {result:?}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

#[test]
fn template_provenance_covers_only_selected_bodies_and_rejects_missing_or_duplicate_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "const C:u8=42;fn plain(x:u8)->u8{x}fn a<T>(x:&T)->&T{x}fn b<const N:usize>()->&'static u8{&C}";
    rustc::qualify("template-region-coverage", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = rustgm::body::templates::build(&report, &facts)?;
    rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let analysis = prove(&facts, &templates)?;
    assert_eq!(analysis.bodies.len(), 2);
    for (body, template) in analysis.bodies.iter().zip(&templates.bodies) {
        assert_eq!(body.source, template.body.source);
    }
    let check_contracts =
        |contracts: &rustgm::body::regions::Contracts<'_>,
         templates: &rustgm::body::templates::Templates| {
            rustgm::borrow::regions::check_templates(
                &facts.types,
                &facts.aggregates,
                &facts.lifetimes.variances,
                contracts,
                templates,
            )
        };
    let mut contracts = rustgm::body::regions::collect(&facts)?;
    let missing = templates.bodies[0].body.source;
    contracts.bodies.retain(|body| body.source != missing);
    assert!(check_contracts(&contracts, &templates).is_err());
    let mut contracts = rustgm::body::regions::collect(&facts)?;
    let duplicate = rustgm::body::regions::collect(&facts)?.bodies.remove(0);
    contracts.bodies.push(duplicate);
    assert!(check_contracts(&contracts, &templates).is_err());
    let contracts = rustgm::body::regions::collect(&facts)?;
    let duplicate = rustgm::body::templates::build(&report, &facts)?
        .bodies
        .remove(0);
    templates.bodies.push(duplicate);
    assert!(check_contracts(&contracts, &templates).is_err());
    Ok(())
}

#[test]
fn promotion_is_demanded_cached_and_keeps_possible_cleanup()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn f<T:'static>(x:T)->&'static[T;0]{&[x;0]}fn g<T:'static>()->&'static &'static[T;0]{&&[]}";
    rustc::qualify("template-promotion-facts", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = rustgm::body::templates::build(&report, &facts)?;
    assert!(prove(&facts, &templates).is_err());
    rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    prove(&facts, &templates)?;
    let mut counts = Vec::new();
    for template in &templates.bodies {
        let promotions = template
            .body
            .promotions
            .as_ref()
            .ok_or("missing promotions")?;
        counts.push((promotions.steps, promotions.entries().count()));
        for (id, _) in promotions.entries() {
            assert!(!template.destruction.values[id.0 as usize]);
        }
    }
    assert!(templates.bodies[0].destruction.locals.contains(&true));
    assert!(counts.iter().all(|(steps, count)| *steps > 0 && *count > 0));
    rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    for (template, expected) in templates.bodies.iter().zip(counts) {
        let promotions = template.body.promotions.as_ref().ok_or("lost promotions")?;
        assert_eq!((promotions.steps, promotions.entries().count()), expected);
    }
    assert!(rustgm::body::build(&report, &facts).is_err());
    Ok(())
}

#[test]
fn symbolic_repeat_promotion_has_an_explicit_pending_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("fn f<const N:usize>()->&'static[u8;N]{&[1u8;N]}", true),
        (
            "fn f<T:Copy+'static,const N:usize>(x:T)->&'static[T;N]{&[x;N]}",
            false,
        ),
    ];
    for (index, (source, accepted)) in cases.iter().enumerate() {
        rustc::qualify(
            &format!("template-symbolic-promotion-{index}"),
            source,
            *accepted,
        )?;
        let (report, mut facts) = semantic::analyze(source)?;
        let mut templates = rustgm::body::templates::build(&report, &facts)?;
        let error = rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)
            .err()
            .ok_or("symbolic repeat received fabricated promotion facts")?;
        assert!(
            error
                .to_string()
                .contains("generic repeat promotion requires deferred constant evaluation")
        );
    }
    Ok(())
}

#[test]
fn provenance_alone_does_not_claim_to_prove_generic_loans() -> Result<(), Box<dyn std::error::Error>>
{
    let cases = [
        "fn f<T>(x:T){let r=&x;let y=x;let z=r;}",
        "fn f<T>(x:&mut T)->(&mut T,&mut T){(x,x)}",
        "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let d=D(&mut n);n=1;}",
    ];
    for (index, source) in cases.iter().enumerate() {
        rustc::qualify(
            &format!("template-provenance-pending-loan-{index}"),
            source,
            false,
        )?;
        check(source)?;
    }
    Ok(())
}
