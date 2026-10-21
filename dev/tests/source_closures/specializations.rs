//! Generic closures retain enclosing substitutions through demand and body proof.
use super::{native, rustc, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "inherited-late",
        "fn copy<'a,T:Copy>(x:&'a T)->T{let f=|x:&'a T|*x;f(x)}fn main(){let x=7u8;check(copy(&x)==7);}",
        false,
    ),
    (
        "inherited-early",
        "fn copy<'a,T:Copy+'a>(x:&'a T)->T{let f=|x:&'a T|*x;f(x)}fn main(){let x=900u16;check(copy(&x)==900);}",
        false,
    ),
    (
        "inherited-nested",
        "fn copy<'a,T:Copy>(x:&'a T)->T{let f=|x:&'a T|{let g=|x:&'a T|*x;g(x)};f(x)}fn main(){let x=7u8;check(copy(&x)==7);}",
        false,
    ),
    (
        "inherited-and-elided",
        "fn copy<'a,T:Copy>(x:&'a T)->T{let f=|pair:(&'a T,&u8)|*pair.0;let local=1u8;f((x,&local))}fn main(){let x=7u8;check(copy(&x)==7);}",
        false,
    ),
    (
        "inherited-pointer",
        "fn pointer<'a,T:Copy+'a>()->fn(&'a T)->T{|x|*x}fn main(){let f=pointer::<u8>();let x=7;check(f(&x)==7);}",
        false,
    ),
    (
        "inherited-reference-argument",
        "fn copy<'a,T:Copy>(x:&'a T)->T{let f=|x:&'a T|*x;f(x)}fn main(){let x=7u8;let r=&x;check(*copy(&r)==7);}",
        false,
    ),
    (
        "inherited-unused",
        "fn value<'a,T>()->u8{let f=||7;f()}fn main(){check(value::<u8>()==7);check(value::<u16>()==7);}",
        false,
    ),
    (
        "repeat",
        "fn repeat<T:Copy>(v:T)->[T;2]{let f=|x:T|[x;2];f(v)}fn main(){let a=repeat(7u8);check(a[0]==7&&a[1]==7);}",
        false,
    ),
    (
        "returned-opaque",
        "fn maker<T>()->impl Fn(u8)->u8{|x|x}fn main(){let f=maker::<u8>();let g=maker::<u16>();check(f(7)==7&&g(9)==9);}",
        false,
    ),
    (
        "custom-cleanup",
        "static mut COUNT:u8=0;struct Token;impl Drop for Token{fn drop(&mut self){unsafe{COUNT+=1;}}}fn consume<T>(v:T){let f=|_x:T|{};f(v)}fn main(){consume(Token);check(unsafe{COUNT}==1);}",
        false,
    ),
    (
        "projected-input",
        "trait Pick{type Out;}impl Pick for u8{type Out=u16;}fn id<T:Pick>(x:T::Out)->T::Out{let f=|x:T::Out|x;f(x)}fn main(){check(id::<u8>(900)==900);}",
        false,
    ),
    (
        "projected-explicit-result",
        "trait Pick{type Out;}impl Pick for u8{type Out=u16;}fn id<T:Pick>(x:T::Out)->T::Out{let f=|x:T::Out|->T::Out{x};f(x)}fn main(){check(id::<u8>(900)==900);}",
        false,
    ),
    (
        "projected-contextual",
        "trait Pick{type Out;}impl Pick for u8{type Out=u16;}fn id<T:Pick>(x:T::Out)->T::Out{let f:fn(T::Out)->T::Out=|x|x;f(x)}fn main(){check(id::<u8>(900)==900);}",
        false,
    ),
    (
        "projected-nested",
        "trait Pick{type Out;}impl Pick for u8{type Out=u16;}fn id<T:Pick>(x:T::Out)->T::Out{let f=|x:T::Out|{let g=|x:T::Out|x;g(x)};f(x)}fn main(){check(id::<u8>(900)==900);}",
        false,
    ),
    (
        "projected-partial",
        "trait Pick{type Out;}impl Pick for u8{type Out=u16;}fn id<T:Pick>(x:T::Out)->T::Out{let f=|x:(T::Out,_)|x;f((x,7u8)).0}fn main(){check(id::<u8>(900)==900);}",
        false,
    ),
    (
        "opaque-reference-signature",
        "fn maker<T>()->impl Fn(&u8)->u8{|x|*x}fn main(){let f=maker::<u8>();let x=7;check(f(&x)==7);}",
        false,
    ),
    (
        "identity",
        "fn id<T>(x:T)->T{let f=|value:T|value;f(x)}fn main(){check(id(7u8)==7);check(id(900u16)==900);}",
        false,
    ),
    (
        "copy",
        "fn twice<T:Copy>(x:T)->(T,T){let f=|value:T|(value,value);f(x)}fn main(){let (a,b)=twice(9u16);check(a==9&&b==9);}",
        false,
    ),
    (
        "move",
        "struct Packet{value:u8}fn id<T>(x:T)->T{let f=|value:T|value;f(x)}fn main(){let p=id(Packet{value:7});check(p.value==7);}",
        false,
    ),
    (
        "const-identity",
        "fn size<const N:usize>()->usize{let f=||N;f()}fn main(){check(size::<7>()==7);check(size::<19>()==19);check(size::<7>()==7);}",
        false,
    ),
    (
        "unused-argument",
        "fn value<T>()->u8{let f=||7;f()}fn main(){check(value::<u8>()==7);check(value::<u16>()==7);}",
        false,
    ),
    (
        "nested",
        "fn id<T>(x:T)->T{let f=|a:T|{let g=|b:T|b;g(a)};f(x)}fn main(){check(id(7u8)==7);check(id(900u16)==900);}",
        false,
    ),
    (
        "reified",
        "fn pointer<T>()->fn(T)->T{let f:fn(T)->T=|x|x;f}fn main(){let a=pointer::<u8>();let b=pointer::<u16>();check(a(7)==7);check(b(900)==900);}",
        false,
    ),
    (
        "call-trait",
        "fn apply<T,F:Fn(T)->T>(f:F,x:T)->T{f(x)}fn id<T>(x:T)->T{let f=|x:T|x;apply(f,x)}fn main(){check(id(7u8)==7);check(id(900u16)==900);}",
        false,
    ),
    (
        "type-reference",
        "fn id<T>(x:T)->T{let f=|x:T|x;f(x)}fn main(){let x=7u8;let r=id(&x);check(*r==7);}",
        false,
    ),
    (
        "borrowed-copy",
        "fn copy<T:Copy>(x:T)->T{let f=|x:&T|*x;f(&x)}fn main(){check(copy(7u8)==7);check(copy(900u16)==900);}",
        false,
    ),
    (
        "contextual-borrow",
        "fn id<T>(x:T)->T{let f:fn(T)->T=|value|value;f(x)}fn main(){check(id(7u8)==7);}",
        false,
    ),
    (
        "recursion",
        "fn repeat<T:Copy>(x:T,n:u8)->T{let f=|x:T,n:u8|if n==0{x}else{repeat(x,n-1)};f(x,n)}fn main(){check(repeat(7u8,3)==7);}",
        false,
    ),
];

fn lower(text: &str) -> Result<rustgm::lowir::program::Program, String> {
    let (mut report, mut facts) = semantic::analyze(text)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
}

#[test]
fn independent_generic_closure_programs() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, text, _) in CASES {
        rustc::qualify(
            &format!("closure-instance-{name}"),
            &format!("{}{text}", native::CHECK),
            true,
        )?;
    }
    Ok(())
}

#[test]
fn generic_closure_programs_have_checked_specializations() {
    let failures: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, text, _)| {
            lower(&format!("{}{text}", native::CHECK))
                .err()
                .map(|error| format!("{name}: {error}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn complete_arguments_distinguish_equal_signatures_and_reuse_equal_instances() -> Result<(), String>
{
    let text = "fn value<T>()->u8{let f=||7;f()}fn main(){let _=value::<u8>();let _=value::<u16>();let _=value::<u8>();}";
    let program = lower(text)?;
    let closures: Vec<_> = program
        .symbols
        .iter()
        .filter(|s| s.name.starts_with("rg.closure."))
        .collect();
    assert_eq!(closures.len(), 2);
    assert_ne!(closures[0].name, closures[1].name);
    assert!(closures.iter().all(|symbol| symbol.body.is_some()));
    Ok(())
}

#[test]
fn generic_closures_must_prove_their_own_permissions_and_results()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, text) in [
        (
            "inherited-reference-escape",
            "fn bad<'a,'b,T>(x:&'a T)->&'b T{let f=|x:&'a T|x;f(x)}fn main(){}",
        ),
        (
            "inherited-borrowed-move",
            "fn bad<'a,T>(x:&'a T)->T{let f=|x:&'a T|*x;f(x)}fn main(){}",
        ),
        (
            "different-projection-results",
            "trait Pick{type Out;}fn owner<T:Pick,U:Pick>(x:T::Out)->U::Out{let f=|x:T::Out|x;f(x)}fn main(){}",
        ),
        (
            "projected-result-mismatch",
            "trait Pick{type Out;}fn owner<T:Pick<Out=u16>>(x:T::Out)->u8{let f=|x:T::Out|x;f(x)}fn main(){}",
        ),
        (
            "repeat-without-copy",
            "fn owner<T>(){let _f=|x:T|[x;2];}fn main(){}",
        ),
        (
            "repeat-clone-only",
            "fn owner<T:Clone>(){let _f=|x:T|[x;2];}fn main(){}",
        ),
        (
            "opaque-wrong-input",
            "fn maker<T>()->impl Fn(u8)->u8{|x:u16|7u8}fn main(){}",
        ),
        (
            "opaque-wrong-result",
            "fn maker<T>()->impl Fn(u8)->u8{|x|7u16}fn main(){}",
        ),
    ] {
        rustc::qualify(&format!("closure-instance-{name}"), text, false)?;
        let error = lower(text)
            .err()
            .ok_or("invalid generic closure was accepted")?;
        assert!(
            !error.contains("not implemented") && !error.contains("requires normalization"),
            "{name}: unsupported behavior is not evidence of this semantic rejection: {error}"
        );
    }
    Ok(())
}

#[test]
fn generic_closure_growth_remains_bounded() -> Result<(), Box<dyn std::error::Error>> {
    let source = "fn grow<T>(){let f=||grow::<(T,T)>();f();}fn main(){grow::<u8>();}";
    rustc::qualify("closure-instance-growing-template", source, true)?;
    let error = lower(source)
        .err()
        .ok_or("unbounded closure specialization accepted")?;
    assert!(
        error.contains("recursive generic instantiation exceeds 128 active instances"),
        "{error}"
    );
    Ok(())
}

#[test]
fn inherited_lifetime_frames_keep_late_closure_inputs_distinct() -> Result<(), String> {
    let source = "fn copy<'a,T:Copy>(x:&'a T)->T{let f=|pair:(&'a T,&u8)|*pair.0;let local=1u8;f((x,&local))}fn main(){let x=7u8;let _=copy(&x);}";
    let (_, facts) = semantic::analyze(source)?;
    let bodies = facts.closures.all();
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0].inherited, 1);
    assert_eq!(bodies[0].universals, 2);
    lower(source)?;
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn native_generic_closure_instances() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("parallel/closure-instances/native", CASES)
}
