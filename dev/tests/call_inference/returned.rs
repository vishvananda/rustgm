//! Zero-sized returned function items keep their exact checked callable identity.
use crate::{native_references as native, qualify as rustc, semantic};
const CASES: &[(&str, bool, &str)] = &[
    (
        "retained",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id(x:&S)->&S{x}fn identity<T>(x:T)->T{x}fn main(){let a=S(3);check(identity(id)(&a).0==3);}struct S(u8);
",
    ),
    (
        "generic-const-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn add<const N:u8>(x:u8)->u8{x+N}fn main(){check(identity(add::<3>)(4)==7);}
",
    ),
    (
        "distinct-type-items",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait V{fn value()->u8;}impl V for u8{fn value()->u8{3}}impl V for u16{fn value()->u8{5}}fn value<T:V>()->u8{T::value()}fn identity<T>(x:T)->T{x}fn main(){check(identity(value::<u8>)()==3);check(identity(value::<u16>)()==5);}
",
    ),
    (
        "once-side-effect",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pass<T>(n:&mut u8,x:T)->T{*n+=1;x}fn id(x:u8)->u8{x+1}fn main(){let mut n=0u8;check(pass(&mut n,id)(3)==4);check(n==1);}
",
    ),
    (
        "reified-cast",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id(x:u8)->u8{x+1}fn main(){let f=identity(id) as fn(u8)->u8;check(f(3)==4);}
",
    ),
    (
        "reified-context",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id(x:u8)->u8{x+1}fn main(){let f:fn(u8)->u8=identity(id);check(f(3)==4);}
",
    ),
    (
        "function-pointer",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id(x:u8)->u8{x+1}fn main(){let f:fn(u8)->u8=id;check(identity(f)(3)==4);}
",
    ),
    (
        "unsafe-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}unsafe fn id(x:u8)->u8{x+1}fn main(){check(unsafe{identity(id)(3)}==4);}
",
    ),
    (
        "unsafe-without-block",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}unsafe fn id(x:u8)->u8{x}fn main(){let _=identity(id)(3);}
",
    ),
    (
        "foreign-convention",
        true,
        r#"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}extern "C" fn id(x:u8)->u8{x+1}fn main(){check(identity(id)(3)==4);}
"#,
    ),
    (
        "foreign-rust-pointer",
        false,
        r#"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}extern "C" fn id(x:u8)->u8{x}fn main(){let _:fn(u8)->u8=identity(id);}
"#,
    ),
    (
        "constructor-item",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn identity<T>(x:T)->T{x}fn main(){check(identity(S)(3).0==3);}
",
    ),
    (
        "unit-not-callable",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn main(){identity(())(3);}
",
    ),
    (
        "unit-effect-only",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn main(){identity(());check(true);}
",
    ),
    (
        "unpaid-static-result",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id(x:&u8)->&u8{x}fn main(){let a=3;let _: &'static u8=identity(id)(&a);}
",
    ),
    (
        "crossed-result",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id<'a,'b>(_:&'a u8,b:&'b u8)->&'b u8{b}fn main(){let a=3;let r;{let b=4;r=identity(id)(&a,&b);}let _=*r;}
",
    ),
    (
        "distinct-const-applications",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn add<const N:u8>(x:u8)->u8{x+N}fn main(){check(identity(add::<3>)(4)==7);check(identity(add::<5>)(4)==9);}
",
    ),
    (
        "empty-closure",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn main(){check(identity(||3)()==3);}
",
    ),
    (
        "captured-closure",
        true,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn main(){let a=3;check(identity(||a)()==3);}
",
    ),
    (
        "wrong-argument",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn identity<T>(x:T)->T{x}fn id(x:u8)->u8{x}fn main(){let _=identity(id)(true);}
",
    ),
    (
        "distinct-items-no-coercion",
        false,
        r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn choose<T>(b:bool,a:T,c:T)->T{if b{a}else{c}}fn id(x:u8)->u8{x}fn other(x:u8)->u8{x+1}fn main(){let _=choose(true,id,other)(3);}
",
    ),
];
#[test]
fn returned_items_preserve_exact_callable_types() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, source) in CASES {
        rustc::qualify(
            &format!("parallel/pa18-returned-items/source/{name}"),
            source,
            accepted,
        )?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn returned_item_producers_execute_once() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|case| case.1)
        .map(|&(name, _, source)| {
            Ok((
                name,
                source
                    .strip_prefix(native::CHECK)
                    .ok_or("missing native check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/pa18-returned-items/native", &cases)
}
