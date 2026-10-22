//! Callable values retain complete instance identities through constants and promotion.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    sema::values::{Evaluator, View},
    types::{Kind, instance::Instance},
};

#[test]
fn constants_keep_type_and_const_instances_including_constructor_values()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn id<T>(x:T)->T{x}fn n<const N:u8>()->u8{N}struct S<T>(T);\
      const A:fn(u8)->u8=id::<u8>;const B:fn(u16)->u16=id::<u16>;\
      const C:fn()->u8=n::<7>;const D:fn()->u8=n::<9>;\
      const E:fn(u8)->S<u8>=S::<u8>;const F:fn(u16)->S<u16>=S::<u16>;";
    rustc::qualify("constant-callable-instances", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let definitions = facts
        .initializers
        .iter()
        .map(|i| i.definition)
        .collect::<Vec<_>>();
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let mut instances = Vec::new();
    for definition in definitions {
        let value = evaluator.definition(definition)?;
        let instance = value.as_function().ok_or("lost callable instance")?;
        assert!(
            matches!(rustgm::sema::values::Reader::new(evaluator.facts()).view(value)?, View::Function(id) if id == instance)
        );
        instances.push(instance);
    }
    assert_eq!(instances.len(), 6);
    for pair in instances.as_chunks::<2>().0 {
        assert_ne!(pair[0], pair[1]);
        let Some(Kind::FunctionItem {
            item: first,
            arguments: a,
            signature: first_sig,
        }) = facts.types.kind(pair[0].ty())
        else {
            return Err("lost first item".into());
        };
        let Some(Kind::FunctionItem {
            item: second,
            arguments: b,
            signature: second_sig,
        }) = facts.types.kind(pair[1].ty())
        else {
            return Err("lost second item".into());
        };
        assert_eq!(first, second);
        assert_ne!(a, b);
        if pair[0] != instances[2] {
            assert_ne!(first_sig, second_sig);
        }
    }
    assert!(Instance::new(&facts.types, rustgm::types::TypeId::UNIT).is_err());
    Ok(())
}

#[test]
fn promoted_function_values_retain_their_selected_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn id<T>(x:T)->T{x}fn f<U>(){let a:&'static fn(u8)->u8=&(id::<u8> as fn(u8)->u8);let b:&'static fn(u16)->u16=&(id::<u16> as fn(u16)->u16);}";
    rustc::qualify("promoted-callable-instances", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = rustgm::body::templates::build(&report, &facts)?;
    rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let instances = templates
        .bodies
        .iter()
        .flat_map(|t| t.body.promotions.iter())
        .flat_map(rustgm::body::promote::Facts::entries)
        .filter_map(|(_, value)| value.as_function())
        .collect::<Vec<_>>();
    assert_eq!(instances.len(), 2);
    assert_ne!(instances[0], instances[1]);
    Ok(())
}

use crate::native_references as native;
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn constructor_function_constants_reify_and_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "constant-constructor-callables",
        &[
            (
                "tuple-struct",
                "struct S(u8);const C:fn(u8)->S=S;fn main(){let a=C(7);check(a.0==7);}",
                false,
            ),
            (
                "enum-variant",
                "enum E{A(u8),B}const C:fn(u8)->E=E::A;fn main(){let a=C(9);check(match a{E::A(n)=>n==9,E::B=>false});}",
                false,
            ),
            (
                "promoted-constructor",
                "struct S(u8);fn main(){let c:&'static fn(u8)->S=&(S as fn(u8)->S);let a=(*c)(11);check(a.0==11);}",
                false,
            ),
        ],
    )
}

fn compile_instances(source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    let lowir = rustgm::lower::instances::build(&report, &mut facts, entry)?;
    let mut bytes = Vec::new();
    rustgm::lowir::writer::write(&lowir, &mut bytes)?;
    Ok(bytes)
}
const INSTANCES: &[(&str, &str, bool)] = &[
    (
        "inferred-item",
        "fn id<T>(x:T)->T{x}fn main(){let f=id;check(f(15u8)==15);}",
        false,
    ),
    (
        "scalar-const-kinds",
        "fn f<const B:bool,const C:char>()->u32{if B{C as u32}else{0}}fn main(){check(f::<true,'🦀'>()==129408);check(f::<false,'A'>()==0);}",
        false,
    ),
    (
        "static-pointers",
        "fn f<const N:u8>()->u8{N}static P:[fn()->u8;2]=[f::<7>,f::<9>];fn main(){check(P[0]()==7&&P[1]()==9);}",
        false,
    ),
    (
        "pointer-backing",
        "fn f<const N:u8>()->u8{N}const P:&'static (fn()->u8,fn()->u8)=&(f::<7>,f::<9>);fn main(){check(P.0()==7&&P.1()==9);}",
        false,
    ),
    (
        "returned-pointer",
        "fn id<T>(x:T)->T{x}fn select<T>()->fn(T)->T{id::<T>}fn main(){let p=select::<u16>();check(p(300)==300);}",
        false,
    ),
    (
        "deferred-sum",
        "fn f<const N:u8>()->&'static u8{&(N+1)}fn main(){check(*f::<7>()==8);}",
        false,
    ),
    (
        "deferred-overflow",
        "fn f<const N:u8>()->&'static u8{&(N+1)}fn main(){let _=f::<255>();}",
        true,
    ),
    (
        "negate-overflow",
        "fn f<const N:i8>()->&'static i8{&(-N)}fn main(){let _=f::<{-128}>();}",
        true,
    ),
    (
        "borrowed-index",
        "fn f<const N:usize,const I:usize>()->&'static u8{&[3;N][I]}fn main(){check(*f::<4,3>()==3);}",
        false,
    ),
    (
        "borrowed-empty",
        "fn f<const N:usize,const I:usize>()->&'static u8{&[3;N][I]}fn main(){let _=f::<0,0>();}",
        true,
    ),
    (
        "borrowed-past-end",
        "fn f<const N:usize,const I:usize>()->&'static u8{&[3;N][I]}fn main(){let _=f::<4,4>();}",
        true,
    ),
    (
        "known-divisor",
        "fn f<const N:u8>()->&'static u8{&(N/2)}fn main(){check(*f::<255>()==127);}",
        false,
    ),
    (
        "widening",
        "fn f<const N:u8>()->&'static u16{&(N as u16+1)}fn main(){check(*f::<255>()==256);}",
        false,
    ),
    (
        "nested-reference",
        "fn f<const N:u8>()->&'static &'static u8{&&N}fn main(){check(**f::<7>()==7);check(**f::<9>()==9);}",
        false,
    ),
    (
        "identity",
        "fn id<T>(x:T)->T{x}fn main(){check(id(7u8)==7);check(id(300u16)==300);check(id(9u8)==9);}",
        false,
    ),
    (
        "recursive",
        "fn f<T>(x:T,n:u8)->T{if n==0{x}else{f(x,n-1)}}fn main(){check(f(7u8,4)==7);}",
        false,
    ),
    (
        "mutual",
        "fn a<T>(x:T,n:u8)->T{if n==0{x}else{b(x,n-1)}}fn b<T>(x:T,n:u8)->T{a(x,n)}fn main(){check(a(9u16,5)==9);}",
        false,
    ),
    (
        "nested",
        "fn id<T>(x:T)->T{x}fn f<T>(x:T)->T{id(x)}fn main(){check(f(11u32)==11);}",
        false,
    ),
    (
        "const-repeat",
        "fn f<const N:usize>()->[u8;N]{[7;N]}fn main(){let a=f::<3>();let b=f::<5>();check(a[2]==7&&b[4]==7);}",
        false,
    ),
    (
        "constant-pointers",
        "fn f<const N:u8>()->u8{N}const P:(fn()->u8,fn()->u8)=(f::<7>,f::<9>);fn main(){check(P.0()==7&&P.1()==9);}",
        false,
    ),
    (
        "promoted-pointers",
        "fn f<const N:u8>()->u8{N}fn main(){let p:&'static fn()->u8=&(f::<7> as fn()->u8);let q:&'static fn()->u8=&(f::<9> as fn()->u8);check((*p)()==7&&(*q)()==9);}",
        false,
    ),
    (
        "promoted-backing",
        "fn f<const N:usize>()->&'static [usize;N]{&[N;N]}fn main(){let a=f::<3>();let b=f::<5>();check(a[2]==3&&b[4]==5);}",
        false,
    ),
    (
        "unused-growing",
        "fn f<T>(){f::<(T,T)>();}fn main(){let _=f::<u8>;check(true);}",
        false,
    ),
    (
        "borrowed-type",
        "fn id<T>(x:T)->T{x}fn f<'a>(x:&'a u8)->&'a u8{id(x)}fn main(){let n=11;check(*f(&n)==11);}",
        false,
    ),
    (
        "late-binder",
        "fn id<'a,T>(x:&'a T)->&'a T{x}fn f<'a,'b>(x:&'a &'b u8)->&'a &'b u8{id(x)}fn main(){let n=13;let p=&n;check(**f(&p)==13);}",
        false,
    ),
];

#[test]
fn demanded_instances_lower_after_template_and_individual_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, _) in INSTANCES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(name, &source, true)?;
        let bytes = compile_instances(&source).map_err(|e| format!("{name}: {e}"))?;
        assert!(!bytes.is_empty());
        assert_eq!(bytes, compile_instances(&source)?, "{name}");
    }
    Ok(())
}
#[test]
fn demanded_growth_is_bounded_but_independent_instances_are_not_depth()
-> Result<(), Box<dyn std::error::Error>> {
    let growing = "fn f<T>(){f::<(T,T)>();}fn main(){f::<u8>();}";
    assert!(
        compile_instances(growing)
            .err()
            .ok_or("growing instance was accepted")?
            .to_string()
            .contains("128 active instances")
    );
    let invalid = "fn bad<T>(x:T){let a=x;let b=x;}fn main(){}";
    rustc::qualify("unused-invalid-instance", invalid, false)?;
    assert!(compile_instances(invalid).is_err());
    let mut many = String::from("fn f<const N:usize>()->usize{N}fn main(){");
    for n in 0..160 {
        use std::fmt::Write;
        write!(&mut many, "let _=f::<{n}>();")?;
    }
    many.push('}');
    compile_instances(&many)?;
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn demanded_instances_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases_with("demanded-instances", INSTANCES, |source, _| {
        compile_instances(source)
    })
}
