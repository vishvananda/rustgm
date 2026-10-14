//! Generic constructor factories preserve nominal identities, fields and lifetime proofs.
#[expect(
    dead_code,
    reason = "this suite uses the typed renderer rather than the CLI harness"
)]
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "explicit-reference-type",
        "struct S<T>(T);fn wrap<'a>(x:&'a u8)->S<&'a u8>{S::<&'a u8>(x)}fn main(){let n=22;check(*wrap(&n).0==22);}",
        false,
    ),
    (
        "owned-transfer",
        "struct D(u8);struct S<T>(T,T);fn main(){let s=S(D(23),D(24));check(s.0.0==23&&s.1.0==24);}",
        false,
    ),
    (
        "zero-sized-constructor",
        "struct S<const N:usize>();fn main(){let f:fn()->S<3>=S::<3>;let g:fn()->S<5>=S::<5>;let a=f();let b=g();check(true);}",
        false,
    ),
    (
        "tuple-types",
        "struct S<T>(T);fn main(){let a=S(7u8);let b=S(300u16);check(a.0==7&&b.0==300);}",
        false,
    ),
    (
        "constructor-pointer",
        "struct S<T>(T);fn main(){let f:fn(u16)->S<u16>=S;let a=f(300);check(a.0==300);}",
        false,
    ),
    (
        "inferred-item",
        "struct S<T>(T);fn main(){let f=S;let a=f(9u8);check(a.0==9);}",
        false,
    ),
    (
        "const-array",
        "struct S<T,const N:usize>([T;N]);fn main(){let a=S([3u16,5,9]);let b=S([4u8;5]);check(a.0[2]==9&&b.0[4]==4);}",
        false,
    ),
    (
        "enum-variants",
        "enum E<T>{A(T),B(T,T),C}fn main(){let f:fn(u16,u16)->E<u16>=E::B;let a=f(300,400);check(match a{E::B(x,y)=>x+y==700,_=>false});let b=E::A(7u8);check(match b{E::A(x)=>x==7,_=>false});}",
        false,
    ),
    (
        "constant-pointer",
        "struct S<T>(T);const C:fn(u16)->S<u16>=S;fn main(){let a=C(500);check(a.0==500);}",
        false,
    ),
    (
        "static-pointer",
        "struct S<T>(T);static C:fn(u16)->S<u16>=S;fn main(){let a=C(501);check(a.0==501);}",
        false,
    ),
    (
        "promoted-pointer",
        "struct S<T>(T);fn main(){let p:&'static fn(u16)->S<u16>=&(S as fn(u16)->S<u16>);let a=(*p)(502);check(a.0==502);}",
        false,
    ),
    (
        "returned-pointer",
        "struct S<T>(T);fn factory<T>()->fn(T)->S<T>{S::<T>}fn main(){let a=factory::<u16>()(503);check(a.0==503);}",
        false,
    ),
    (
        "nested-demand",
        "struct S<T>(T);fn wrap<T>(x:T)->S<T>{S(x)}fn main(){let a=wrap(504u16);check(a.0==504);}",
        false,
    ),
    (
        "reference-argument",
        "struct S<T>(T);fn wrap<'a>(x:&'a u8)->S<&'a u8>{S(x)}fn main(){let n=17;check(*wrap(&n).0==17);}",
        false,
    ),
    (
        "declared-lifetime",
        "struct S<'a,T>(&'a T);fn wrap<'a,T>(x:&'a T)->S<'a,T>{S(x)}fn main(){let n=18;check(*wrap(&n).0==18);}",
        false,
    ),
    (
        "nested-lifetimes",
        "struct S<'a,'b,T:'b>(&'a &'b T);fn wrap<'a,'b>(x:&'a &'b u8)->S<'a,'b,u8>{S(x)}fn main(){let n=19;let p=&n;check(**wrap(&p).0==19);}",
        false,
    ),
    (
        "bounded-parameter",
        "struct S<T:Copy>(T,T);fn make<T:Copy>(x:T)->S<T>{S(x,x)}fn main(){let a=make(20u8);check(a.0==20&&a.1==20);}",
        false,
    ),
    (
        "empty-const-array",
        "struct S<T,const N:usize>([T;N]);fn main(){let a:S<u8,0>=S([]);let b=S([21u8]);check(b.0[0]==21);}",
        false,
    ),
];

fn compile(source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let lowir = rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let mut bytes = Vec::new();
    rustgm::lowir::writer::write(&lowir, &mut bytes)?;
    Ok(bytes)
}

#[test]
fn demanded_constructor_bodies_have_independent_proofs() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, _) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(name, &source, true)?;
        let bytes = compile(&source).map_err(|e| format!("{name}: {e}"))?;
        assert!(!bytes.is_empty());
        assert_eq!(bytes, compile(&source)?, "{name}");
    }
    Ok(())
}

#[test]
fn constructor_instances_cannot_escape_borrowed_storage_or_duplicate_moves()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "escaping-constructor",
            "struct S<T>(T);fn bad<'a>()->S<&'a u8>{let n=7;S(&n)}fn main(){}",
        ),
        (
            "escaping-nominal-lifetime",
            "struct S<'a,T>(&'a T);fn bad<'a>()->S<'a,u8>{let n=7;S(&n)}fn main(){}",
        ),
        (
            "constructor-move",
            "struct D(u8);struct S<T>(T,T);fn main(){let d=D(7);let s=S(d,d);}",
        ),
        (
            "constructor-bound",
            "struct D(u8);struct S<T:Copy>(T);fn main(){let s=S(D(7));}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        assert!(compile(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn constructor_instances_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases_with("constructor-instances", CASES, |source, _| compile(source))
}

#[test]
fn implicit_constructor_templates_share_nominal_assumptions_and_retain_moves()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::body;
    let source = "struct Pair<T>(T,T);enum E<T:Copy>{A(T),B(T,T)}fn main(){let _=Pair::<u8>;let _=E::<u8>::A;let _=E::<u8>::B;}";
    rustc::qualify("implicit-constructor-templates", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    assert!(templates.bodies.is_empty());
    let mut contracts = body::regions::collect(&facts)?.bodies;
    let ordinary = contracts.len();
    body::templates::constructors(&report, &mut facts, &mut templates, &mut contracts)?;
    assert_eq!(templates.bodies.len(), 3);
    assert_eq!(contracts.len(), ordinary + 3);
    let pair = &templates.bodies[0];
    let owned = pair
        .body
        .owned
        .as_ref()
        .ok_or("missing generic field transfer scopes")?;
    assert_eq!(owned.created.len(), 3);
    assert_eq!(owned.returned.len(), 1);
    assert_eq!(pair.destruction.locals, [true, true]);
    assert!(std::sync::Arc::ptr_eq(
        &templates.bodies[1].environment,
        &templates.bodies[2].environment
    ));
    for template in &templates.bodies[1..] {
        assert!(template.body.owned.is_none());
        assert!(template.destruction.locals.iter().all(|needed| !needed));
    }
    for contract in &contracts[ordinary..] {
        assert_eq!(contract.parameters.len(), 1);
    }
    Ok(())
}

#[test]
fn unused_implicit_factories_need_no_synthetic_body() -> Result<(), Box<dyn std::error::Error>> {
    let source = "struct Pair<T>(T,T);enum E<T:Copy>{A(T),B(T,T)}fn main(){}";
    rustc::qualify("unused-implicit-factories", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = rustgm::body::templates::build(&report, &facts)?;
    let mut contracts = rustgm::body::regions::collect(&facts)?.bodies;
    rustgm::body::templates::constructors(&report, &mut facts, &mut templates, &mut contracts)?;
    assert!(templates.bodies.is_empty());
    assert_eq!(contracts.len(), 1);
    compile(source)?;
    Ok(())
}
