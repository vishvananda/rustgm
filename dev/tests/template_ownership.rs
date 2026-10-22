use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "repeat-constant-owned",
        "struct D;impl Drop for D{fn drop(&mut self){}}const VALUE:D=D;fn f<const N:usize>()->[D;N]{[VALUE;N]}",
        true,
    ),
    (
        "repeat-rvalue-owned",
        "struct D;impl Drop for D{fn drop(&mut self){}}fn f<const N:usize>()->[D;N]{[D;N]}",
        false,
    ),
    (
        "repeat-block-parameter",
        "fn f<T:Copy,const N:usize>(x:T)->[T;N]{[x;{N}]}",
        true,
    ),
    (
        "repeat-wrong-const-type",
        "fn f<const N:u8>(){let x=[1u8;N];}",
        false,
    ),
    (
        "repeat-local-shadow",
        "fn f<const N:usize>(){let N=3;let x=[1u8;N];}",
        false,
    ),
    (
        "repeat-const-expression",
        "fn f<T:Copy,const N:usize>(x:T){let y=[x;N+1];}",
        false,
    ),
    ("identity", "fn f<T>(x:T)->T{x}", true),
    ("twice", "fn f<T>(x:T)->(T,T){(x,x)}", false),
    ("copy", "fn f<T:Copy>(x:T)->(T,T){(x,x)}", true),
    ("clone", "fn f<T:Clone>(x:T)->(T,T){(x,x)}", false),
    (
        "unused-invalid",
        "fn f<T>(x:T){let y=x;let z=x;}fn main(){}",
        false,
    ),
    (
        "tuple-copy",
        "fn f<T,U>(x:(T,U))->((T,U),(T,U))where(T,U):Copy{(x,x)}",
        true,
    ),
    (
        "tuple-element",
        "fn f<T,U>(x:(T,U))->(T,T)where(T,U):Copy{(x.0,x.0)}",
        false,
    ),
    ("reinitialize", "fn f<T>(mut x:T)->T{let y=x;x=y;x}", true),
    ("immutable-write", "fn f<T>(x:T,y:T)->T{x=y;x}", false),
    (
        "conditional-init",
        "fn f<T>(value:T,c:bool)->T{let x:T;if c{x=value;}x}",
        false,
    ),
    (
        "both-branches",
        "fn f<T>(x:T,c:bool)->T{if c{x}else{x}}",
        true,
    ),
    (
        "branch-move",
        "fn f<T>(x:T,c:bool)->T{if c{let y=x;}x}",
        false,
    ),
    ("loop-move", "fn f<T>(x:T,c:bool){while c{let y=x;}}", false),
    (
        "loop-restore",
        "fn f<T>(mut x:T,c:bool)->T{while c{let y=x;x=y;}x}",
        true,
    ),
    ("loop-break", "fn f<T>(x:T)->T{loop{break x;}}", true),
    ("repeat-zero", "fn f<T>(x:T)->[T;0]{[x;0]}", true),
    ("repeat-one", "fn f<T>(x:T)->[T;1]{[x;1]}", true),
    ("repeat-two", "fn f<T>(x:T)->[T;2]{[x;2]}", false),
    (
        "repeat-symbolic",
        "fn f<T:Copy,const N:usize>(x:T)->[T;N]{[x;N]}",
        true,
    ),
    (
        "repeat-symbolic-unbounded",
        "fn f<T,const N:usize>(x:T)->[T;N]{[x;N]}",
        false,
    ),
    ("shared", "fn f<T>(x:&T)->(&T,&T){(x,x)}", true),
    ("borrowed-move", "fn f<T>(x:&mut T)->T{*x}", false),
    ("borrowed-copy", "fn f<T:Copy>(x:&T)->T{*x}", true),
    (
        "reference-assumption",
        "fn f<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{let a=x;let b=x;}",
        true,
    ),
    (
        "drop-field",
        "struct N;struct D(N);impl Drop for D{fn drop(&mut self){}}fn f<T>(d:D){let n=d.0;}",
        false,
    ),
    (
        "drop-copy-field",
        "struct D(u8);impl Drop for D{fn drop(&mut self){}}fn f<T>(d:D){let n=d.0;}",
        true,
    ),
    ("wildcard", "fn f<T>(x:T)->T{let _=x;x}", true),
    (
        "partial-fields",
        "fn f<T>(x:(T,T))->T{let(y,_)=x;let z=x.1;y}",
        true,
    ),
    (
        "partial-whole",
        "fn f<T>(x:(T,T))->(T,T){let(y,_)=x;x}",
        false,
    ),
    (
        "discard-call",
        "fn make<T>(x:T)->T{x}fn f<T>(x:T){make(x);}",
        true,
    ),
];

#[test]
fn template_move_proofs_do_not_authorize_pending_mutable_loans()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = [
        "fn f<T>(x:&mut T)->(&mut T,&mut T){(x,x)}",
        "fn f<'a,T:'a>(x:&'a mut T)->(&'a mut T,&'a mut T)where &'a mut T:Copy{(x,x)}",
        "fn take<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{}fn f<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{take::<'a,T>(x);take::<'a,T>(x);}",
    ];
    for (index, source) in sources.iter().enumerate() {
        rustc::qualify(&format!("template-pending-loans-{index}"), source, false)?;
        check(source)?;
        let (report, facts) = semantic::analyze(source)?;
        assert!(rustgm::body::build(&report, &facts).is_err());
    }
    Ok(())
}

#[test]
fn template_environments_share_unchanged_assumptions_and_retain_free_aliases()
-> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    let source = "fn plain<T:Copy>(x:T)->T{x}fn aliased<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{let a=x;let b=x;}";
    rustc::qualify("template-environments", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let templates = rustgm::body::templates::build(&report, &facts)?;
    ownership(&facts, &templates).map_err(|e| e.message)?;
    for function in &facts.functions {
        let original = facts
            .generics
            .environments
            .get(&function.definition)
            .ok_or("missing declaration environment")?;
        let body = facts
            .generics
            .body_environment(function.definition)
            .ok_or("missing body environment")?;
        let name = report
            .package
            .names
            .get(report.resolution.definitions[function.definition.0 as usize].name);
        assert_eq!(Arc::ptr_eq(original, body), name == "plain");
    }
    Ok(())
}

fn check(source: &str) -> Result<(), String> {
    let (report, facts) = semantic::analyze(source)?;
    let templates = rustgm::body::templates::build(&report, &facts).map_err(|e| e.to_string())?;
    ownership(&facts, &templates)
        .map(|_| ())
        .map_err(|e| e.message.to_owned())
}

#[test]
fn unused_generic_bodies_obey_their_own_move_assumptions() -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        let result = check(source);
        if result.is_ok() != *accepted {
            failures.push(format!(
                "{name}: wanted {accepted}; {}",
                result.err().unwrap_or_default()
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

#[test]
fn declared_template_move_outcomes_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("template-move-{name}"), source, *accepted)?;
    }
    Ok(())
}

#[test]
fn possible_cleanup_retains_discarded_forwarded_and_zero_repeat_operands()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn make<T>(x:T)->T{x}fn discarded<T>(x:T){make(x);}fn forwarded<T>(x:T)->T{{make(x)}}fn zero<T>(x:T)->[T;0]{[make(x);0]}fn copied<T:Copy>(x:T)->T{make(x)}";
    rustc::qualify("template-owned-scopes", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let templates = rustgm::body::templates::build(&report, &facts)?;
    ownership(&facts, &templates).map_err(|e| e.message)?;
    assert_eq!(templates.bodies.len(), 5);
    for template in &templates.bodies {
        use rustgm::body::ir::{Owner, Rvalue};
        let Owner::Function { definition, .. } = template.body.owner else {
            return Err("not a function".into());
        };
        let name = report
            .package
            .names
            .get(report.resolution.definitions[definition.0 as usize].name);
        if name == "copied" {
            assert!(template.body.owned.is_none());
            assert!(!template.destruction.values.contains(&true));
            continue;
        }
        let owned = template
            .body
            .owned
            .as_ref()
            .ok_or("missing possible cleanup scopes")?;
        assert!(!owned.created.is_empty());
        assert!(template.destruction.locals.contains(&true));
        if name == "discarded" || name == "zero" {
            assert!(owned.returned.is_empty());
        } else {
            assert_eq!(owned.returned.len(), 1);
        }
        for (index, value) in template.body.values.iter().enumerate() {
            if matches!(value.kind, Rvalue::Call { .. }) {
                assert!(template.destruction.values[index]);
            }
            if matches!(value.kind, Rvalue::Repeat { .. }) {
                assert!(!template.destruction.values[index]);
            }
        }
    }
    let error = rustgm::body::build(&report, &facts)
        .err()
        .ok_or("template received a concrete CFG")?;
    assert!(
        error
            .to_string()
            .contains("parameterized ownership and destruction")
    );
    Ok(())
}

fn ownership(
    facts: &rustgm::sema::model::Declarations,
    templates: &rustgm::body::templates::Templates,
) -> Result<Vec<rustgm::borrow::ownership::BodyFacts>, rustgm::borrow::ownership::Error> {
    let custom = (!facts.destructors.implementations().is_empty())
        .then_some(|item| facts.destructors.item(item).is_some());
    rustgm::borrow::ownership::templates::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        custom.as_ref(),
        templates,
    )
}
