//! Closure bodies infer through their call sites while retaining separate lexical control.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    ("empty", "fn main(){let f=||{};f();}", true),
    (
        "annotated",
        "fn main(){let f=|x:u8|->u8{x+1};let _:u8=f(7);}",
        true,
    ),
    (
        "inferred-input",
        "fn main(){let f=|x|x;let _:u8=f(7);}",
        true,
    ),
    (
        "inferred-output",
        "fn main(){let f=||7;let _:u8=f();}",
        true,
    ),
    ("default-output", "fn main(){let f=||7;let _=f();}", true),
    (
        "parameter-pattern",
        "fn main(){let f=|(a,b):(u8,u16)|b;let _:u16=f((1,2));}",
        true,
    ),
    (
        "nested",
        "fn main(){let f=||{let g=|x|x;g(7u8)};let _:u8=f();}",
        true,
    ),
    (
        "return-isolated",
        "fn outer()->u8{let f=||->u16{return 7;};let _:u16=f();3}fn main(){}",
        true,
    ),
    (
        "move-empty",
        "fn main(){let f=move|x:u8|x;let _=f(7);let _=f(8);}",
        true,
    ),
    (
        "generic-bound",
        "fn apply<F:Fn(u8)->u8>(f:F)->u8{f(7)}fn main(){let _=apply(|x|x);}",
        true,
    ),
    ("wrong-arity", "fn main(){let f=|x:u8|x;f();}", false),
    ("wrong-input", "fn main(){let f=|x:u8|x;f(7u16);}", false),
    (
        "wrong-result",
        "fn main(){let f=|x:u8|->u16{x};f(7);}",
        false,
    ),
    (
        "one-signature",
        "fn main(){let f=|x|x;f(1u8);f(2u16);}",
        false,
    ),
    (
        "distinct-identity",
        "fn main(){let mut f=|x:u8|x;f=|x:u8|x;}",
        false,
    ),
    (
        "return-mismatch",
        "fn main(){let f=||->u8{return 7u16;};f();}",
        false,
    ),
    (
        "break-isolated",
        "fn main(){loop {let f=||{break;};break;}}",
        false,
    ),
    (
        "generic-owner",
        "fn apply<T>(x:T)->T{let f=|y:T|y;f(x)}fn main(){let _:u8=apply(7);}",
        true,
    ),
    (
        "const-owner",
        "fn size<const N:usize>()->usize{let f=||N;f()}fn main(){let _=size::<7>();}",
        true,
    ),
    (
        "generic-owner-argument",
        "fn make<T>(){let f=||7u8;let _=f();}fn main(){make::<u16>();}",
        true,
    ),
    (
        "nested-return",
        "fn main(){let f=|| ||7;let _:u8=f()();}",
        true,
    ),
    (
        "conditional-result",
        "fn main(){let f=|x:bool|if x{7u8}else{9};let _=f(true);}",
        true,
    ),
    (
        "local-loop",
        "fn main(){let f=||loop{break 7u8;};let _:u8=f();}",
        true,
    ),
    (
        "parameter-mutation",
        "fn main(){let f=|mut x:u8|{x+=1;x};let _=f(7);let _=f(8);}",
        true,
    ),
    (
        "named-call",
        "fn id(x:u8)->u8{x}fn main(){let f=|x|id(x);let _:u8=f(7);}",
        true,
    ),
    (
        "local-constraint",
        "fn main(){let f=|x|{let y:u16=x;y};let _:u16=f(7);}",
        true,
    ),
    (
        "closure-consumes-parameter",
        "struct P(u8);fn main(){let f=|x:P|x;let _=f(P(7));let _=f(P(8));}",
        true,
    ),
    (
        "static-use",
        "static X:u8=7;fn main(){let f=||X;let _:u8=f();}",
        true,
    ),
    (
        "constant-use",
        "const X:u8=7;fn main(){let f=||X;let _:u8=f();}",
        true,
    ),
    (
        "nested-return-isolation",
        "fn main(){let f=||{let g=||->u16{return 9;};let _=g();7u8};let _:u8=f();}",
        true,
    ),
    ("unresolved-unused-input", "fn main(){let _f=|x|x;}", false),
    (
        "annotation-conflict",
        "fn main(){let f=|x:u8|{let y:u16=x;y};let _=f(7);}",
        false,
    ),
    (
        "branch-conflict",
        "fn main(){let f=|x:bool|if x{7u8}else{9u16};let _=f(true);}",
        false,
    ),
    (
        "nested-bad-return",
        "fn main(){let f=||{let g=||->u8{return 9u16;};let _=g();7u8};let _=f();}",
        false,
    ),
];

#[test]
fn source_bodies_retain_constraints_and_control_ownership() {
    for &(name, source, accepted) in CASES {
        let result = semantic::analyze(source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {}",
            result.err().unwrap_or_default()
        );
    }
}
#[test]
fn independent_source_body_expectations() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("pa24-body-{name}"), source, accepted)?;
    }
    Ok(())
}
#[test]
fn closure_signatures_have_distinct_retained_body_identities() -> Result<(), String> {
    let (_, facts) =
        semantic::analyze("fn main(){let a=|x:u8|x;let b=||3u16;let _=a(7);let _=b();}")?;
    assert_eq!(facts.closures.all().len(), 2);
    let a = facts.closures.all()[0];
    let b = facts.closures.all()[1];
    assert_ne!(a.identity, b.identity);
    assert_ne!(a.body, b.body);
    assert_ne!(a.signature, b.signature);
    assert!(facts.closures.get(a.source).is_some());
    Ok(())
}

#[test]
fn nested_body_work_tracks_source_size() -> Result<(), String> {
    let mut previous = None;
    for count in [8_usize, 16, 32, 48] {
        let mut source = String::from("fn main(){");
        source.push_str(&"let f=||{".repeat(count));
        source.push_str("7u8");
        source.push_str(&"};f()".repeat(count));
        source.push_str(";}");
        let (_, facts) = semantic::analyze(&source)?;
        assert_eq!(facts.closures.all().len(), count);
        let work = facts.body_work.inference_steps
            + facts.body_work.canonical_steps
            + facts.body_work.steps;
        if let Some((prior_count, prior_work)) = previous {
            assert!(
                work * prior_count <= prior_work * count + count * 100,
                "closure work grew faster than source: {prior_count}/{prior_work}, {count}/{work}"
            );
        }
        assert!(work < 5_000 * count, "{count}: {work}");
        println!("nested closures={count} inference_and_body_work={work}");
        previous = Some((count, work));
    }
    Ok(())
}
#[test]
fn unused_enclosing_parameters_remain_in_closure_identity() -> Result<(), String> {
    let (_, facts) =
        semantic::analyze("fn make<T,const N:usize>(){let f=||7u8;let _=f();}fn main(){}")?;
    let closure = facts.closures.all()[0];
    let ty = facts
        .node(closure.source)
        .and_then(|n| n.ty)
        .ok_or("missing closure type")?;
    let Some(rustgm::types::Kind::Closure { arguments, .. }) = facts.types.kind(ty) else {
        return Err("missing closure identity".into());
    };
    assert_eq!(
        facts
            .types
            .arguments(arguments)
            .ok_or("missing closure arguments")?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn adjacent_bodies_do_not_copy_prior_local_constraints() -> Result<(), String> {
    use std::fmt::Write;
    let mut previous = None;
    for count in [8_usize, 32, 128, 512] {
        let mut source = String::from("fn main(){");
        for index in 0..count {
            write!(source, "let f{index}=|x:u8|x;let _=f{index}(7);").map_err(|e| e.to_string())?;
        }
        source.push('}');
        let (_, facts) = semantic::analyze(&source)?;
        assert_eq!(facts.closures.all().len(), count);
        let work = facts.body_work.inference_steps
            + facts.body_work.canonical_steps
            + facts.body_work.steps;
        if let Some((prior_count, prior_work)) = previous {
            assert!(
                work * prior_count <= prior_work * count + count * 100,
                "adjacent closure work grew faster than source: {prior_count}/{prior_work}, {count}/{work}"
            );
        }
        println!("adjacent closures={count} inference_and_body_work={work}");
        previous = Some((count, work));
    }
    Ok(())
}

const NATIVE: &[(&str, &str, bool)] = &[
    (
        "direct",
        "fn main(){let f=|x:u8|x+1;check(f(7)==8);check(f(9)==10);}",
        false,
    ),
    (
        "inferred",
        "fn main(){let f=|x|x;let y:u16=f(777);check(y==777);}",
        false,
    ),
    (
        "nested",
        "fn main(){let f=||{let g=|x:u8|x+2;g(7)};check(f()==9);}",
        false,
    ),
    (
        "returned",
        "fn main(){let f=|| ||7u8;check(f()()==7);}",
        false,
    ),
    (
        "tuple",
        "fn main(){let f=|(a,b):(u8,u16)|(b,a);let p=f((7,999));check(p.0==999&&p.1==7);}",
        false,
    ),
    (
        "named-call",
        "fn id(x:u8)->u8{x+1}fn main(){let f=|x|id(x);check(f(7)==8);}",
        false,
    ),
    (
        "by-reference",
        "fn main(){let f=|x:u8|x+1;let r=&&f;check(r(7)==8);}",
        false,
    ),
    (
        "mutable-parameter",
        "fn main(){let f=|mut x:u8|{x+=1;x};check(f(7)==8);check(f(8)==9);}",
        false,
    ),
    (
        "generic-call",
        "fn apply<F:Fn(u8)->u8>(f:F)->u8{f(7)+f(8)}fn main(){check(apply(|x|x+1)==17);}",
        false,
    ),
    (
        "once-call",
        "fn apply<F:FnOnce(u8)->u8>(f:F)->u8{f(7)}fn main(){check(apply(|x|x+1)==8);}",
        false,
    ),
    (
        "consumed-parameter",
        "struct P(u8);fn main(){let f=|p:P|p;check(f(P(7)).0==7);check(f(P(8)).0==8);}",
        false,
    ),
    (
        "checked-arithmetic",
        "fn main(){let f=|x:u8|1/x;let _=f(0);}",
        true,
    ),
    (
        "generic-body",
        "fn id<T>(x:T)->T{x}fn main(){let f=|x:u8|id(x);check(f(7)==7);}",
        false,
    ),
    (
        "returned-generic-body",
        "fn id<T>(x:T)->T{x}fn main(){let f=|| ||id(7u8);check(f()()==7);}",
        false,
    ),
    (
        "reference-input",
        "fn main(){let f=|x:&u8|*x;let x=7;check(f(&x)==7);}",
        false,
    ),
    (
        "local-borrow",
        "fn main(){let f=||{let mut x=7u8;let a=&mut x;*a+=1;let b=&mut x;*b};check(f()==8);}",
        false,
    ),
    (
        "return-control",
        "fn outer()->u8{let f=||->u16{return 999;};check(f()==999);7}fn main(){check(outer()==7);}",
        false,
    ),
];
#[test]
fn source_closures_reach_checked_lowir() -> Result<(), String> {
    for &(name, source, _) in NATIVE {
        let (mut report, mut facts) = semantic::analyze(&format!("{}{source}", native::CHECK))
            .map_err(|e| format!("{name} semantics: {e}"))?;
        let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
        rustgm::lower::instances::build(&report, &mut facts, entry)
            .map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn source_closures_execute_checked_bodies() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("pa24-source-bodies/native", NATIVE)
}

#[test]
fn unused_closure_bodies_still_require_ownership_and_lifetime_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "double-move",
            "struct P(u8);fn main(){let _f=|p:P|{let a=p;let b=p;};}",
        ),
        (
            "conflicting-borrows",
            "fn main(){let _f=||{let mut x=7u8;let a=&mut x;let b=&mut x;*a+*b};}",
        ),
        ("dangling-result", "fn main(){let _f=||{let x=7u8;&x};}"),
    ] {
        rustc::qualify(&format!("closure-proof-{name}"), source, false)?;
        let analyzed = semantic::analyze(source);
        if name == "dangling-result" && analyzed.is_err() {
            continue;
        }
        let (mut report, mut facts) = analyzed?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        let result = rustgm::lower::instances::build(&report, &mut facts, entry);
        assert!(result.is_err(), "unchecked closure body: {name}");
    }
    Ok(())
}
