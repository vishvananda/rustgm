//! Callable implementations retain their actual input/output equations and lifetime proof.
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "pointer-shared",
        "fn need<F:Fn(u8)->u16>(){}fn main(){need::<fn(u8)->u16>();}",
        true,
    ),
    (
        "pointer-mutable",
        "fn need<F:FnMut(u8)->u16>(){}fn main(){need::<fn(u8)->u16>();}",
        true,
    ),
    (
        "pointer-once",
        "fn need<F:FnOnce(u8)->u16>(){}fn main(){need::<fn(u8)->u16>();}",
        true,
    ),
    (
        "function-item",
        "fn id(x:u8)->u16{x as u16}fn need<F:Fn(u8)->u16>(_:F){}fn main(){need(id);}",
        true,
    ),
    (
        "item-inference",
        "fn id<T>(x:T)->T{x}fn need<F:Fn(u8)->u8>(_:F){}fn main(){need(id);}",
        true,
    ),
    (
        "item-first-use",
        "fn id<T>(x:T)->T{x}fn need<F:Fn(u8)->u8>(_:F){}fn main(){let f=id;need(f);}",
        true,
    ),
    (
        "pointer-wrong-input",
        "fn need<F:Fn(u8)->u8>(){}fn main(){need::<fn(u16)->u8>();}",
        false,
    ),
    (
        "pointer-wrong-output",
        "fn need<F:Fn(u8)->u8>(){}fn main(){need::<fn(u8)->u16>();}",
        false,
    ),
    (
        "pointer-wrong-arity",
        "fn need<F:Fn(u8,u8)->u8>(){}fn main(){need::<fn(u8)->u8>();}",
        false,
    ),
    (
        "unsafe-pointer",
        "fn need<F:Fn(u8)->u8>(){}fn main(){need::<unsafe fn(u8)->u8>();}",
        false,
    ),
    (
        "unsafe-item",
        "unsafe fn id(x:u8)->u8{x}fn need<F:Fn(u8)->u8>(_:F){}fn main(){need(id);}",
        false,
    ),
    (
        "pointer-quantified",
        "fn need<F:for<'a>Fn(&'a u8)->&'a u8>(){}fn main(){need::<fn(&u8)->&u8>();}",
        true,
    ),
    (
        "item-quantified",
        "fn id(x:&u8)->&u8{x}fn need<F:for<'a>Fn(&'a u8)->&'a u8>(_:F){}fn main(){need(id);}",
        true,
    ),
    (
        "pointer-static-not-universal",
        "fn need<F:for<'a>Fn(&'a u8)->&'a u8>(){}fn main(){need::<fn(&'static u8)->&'static u8>();}",
        false,
    ),
    (
        "pointer-output-identity",
        "fn need<F:for<'a,'b>Fn(&'a u8,&'b u8)->&'a u8>(){}fn main(){need::<for<'a,'b>fn(&'a u8,&'b u8)->&'b u8>();}",
        false,
    ),
    (
        "projected-pointer",
        "trait A{type Out;}impl A for u8{type Out=fn(u8)->u16;}fn need<F:Fn(u8)->u16>(){}fn main(){need::<<u8 as A>::Out>();}",
        true,
    ),
    (
        "shadowed-call-trait",
        "trait Fn<A>{}fn need<F:Fn<u8>>(){}fn main(){need::<fn(u8)->u8>();}",
        false,
    ),
    (
        "unrelated-subject",
        "struct S;fn need<F:Fn()>(){}fn main(){need::<S>();}",
        false,
    ),
    (
        "output-infers-item",
        "fn make<T>()->T{loop{}}fn need<F:Fn()->u8>(_:F){}fn f(){need(make::<_>);}fn main(){}",
        true,
    ),
    (
        "unused-item-parameter",
        "fn item<T>(x:u8)->u8{x}fn need<F:Fn(u8)->u8>(_:F){}fn f(){need(item::<_>);}fn main(){}",
        false,
    ),
    (
        "output-infers-item-negative",
        "fn make<T>()->T{loop{}}fn need<F:Fn()->u8>(_:F){}fn f(){need(make::<u16>);}fn main(){}",
        false,
    ),
    (
        "generic-id-not-polymorphic",
        "fn id<T>(x:T)->T{x}fn need<F:for<'a>Fn(&'a u8)->&'a u8>(_:F){}fn f(){need(id::<_>);}fn main(){}",
        false,
    ),
    (
        "generic-output-no-escape",
        "fn make<T>(_:&u8)->T{loop{}}fn need<F:for<'a>Fn(&'a u8)->&'a u8>(_:F){}fn f(){need(make::<_>);}fn main(){}",
        false,
    ),
    (
        "late-signature-static-use",
        "fn id(x:&u8)->&u8{x}fn need<F:Fn(&'static u8)->&'static u8>(_:F){}fn f(){need(id);}fn main(){}",
        true,
    ),
    (
        "nested-input-binders",
        "fn item(_:for<'a,'b>fn(&'a u8,&'b u8)->&'a u8){}fn need<F:Fn(for<'c,'d>fn(&'c u8,&'d u8)->&'c u8)>(_:F){}fn f(){need(item);}fn main(){}",
        true,
    ),
    (
        "nested-input-wrong-output",
        "fn item(_:for<'a,'b>fn(&'a u8,&'b u8)->&'a u8){}fn need<F:Fn(for<'c,'d>fn(&'c u8,&'d u8)->&'d u8)>(_:F){}fn f(){need(item);}fn main(){}",
        false,
    ),
    (
        "output-infers-callable",
        "fn make<T>()->T{loop{}}fn need<F:Fn()->for<'a>fn(&'a u8)->&'a u8>(_:F){}fn f(){need(make::<_>);}fn main(){}",
        true,
    ),
    (
        "projected-output-inference",
        "trait Out{type T;}impl Out for u8{type T=u16;}fn make<T>()->T{loop{}}fn need<F:Fn()-><u8 as Out>::T>(_:F){}fn f(){need(make::<_>);}fn main(){}",
        true,
    ),
    (
        "shared-output-mismatch",
        "fn make<T>()->(T,T){loop{}}fn need<F:Fn()->(u8,u16)>(_:F){}fn f(){need(make::<_>);}fn main(){}",
        false,
    ),
    (
        "output-infers-pair",
        "fn make<T,U>()->(T,U){loop{}}fn need<F:Fn()->(u8,u16)>(_:F){}fn f(){need(make::<_,_>);}fn main(){}",
        true,
    ),
    (
        "target-feature-callable",
        "#[target_feature(enable=\"sse2\")]fn item(x:u8)->u8{x}fn need<F:Fn(u8)->u8>(_:F){}fn f(){need(item);}fn main(){}",
        false,
    ),
    (
        "target-feature-disabled",
        "#[cfg_attr(any(),target_feature(enable=\"sse2\"))]fn item(x:u8)->u8{x}fn need<F:Fn(u8)->u8>(_:F){}fn f(){need(item);}fn main(){}",
        true,
    ),
    (
        "target-feature-configured",
        "#[cfg_attr(all(),target_feature(enable=\"sse2\"))]fn item(x:u8)->u8{x}fn need<F:FnOnce(u8)->u8>(_:F){}fn f(){need(item);}fn main(){}",
        false,
    ),
    (
        "shared-reference",
        "fn need<F:Fn(u8)->u8>(){}fn f<F:Fn(u8)->u8>(){need::<&F>();}fn main(){}",
        true,
    ),
    (
        "mutable-reference-shared",
        "fn need<F:Fn(u8)->u8>(){}fn f<F:Fn(u8)->u8>(){need::<&mut F>();}fn main(){}",
        false,
    ),
    (
        "shared-reference-mutable",
        "fn need<F:FnMut(u8)->u8>(){}fn f<F:Fn(u8)->u8>(){need::<&F>();}fn main(){}",
        true,
    ),
    (
        "shared-reference-insufficient",
        "fn need<F:FnMut(u8)->u8>(){}fn f<F:FnMut(u8)->u8>(){need::<&F>();}fn main(){}",
        false,
    ),
    (
        "mutable-reference-once",
        "fn need<F:FnOnce(u8)->u8>(){}fn f<F:FnMut(u8)->u8>(){need::<&mut F>();}fn main(){}",
        true,
    ),
    (
        "mutable-reference-once-insufficient",
        "fn need<F:FnOnce(u8)->u8>(){}fn f<F:FnOnce(u8)->u8>(){need::<&mut F>();}fn main(){}",
        false,
    ),
    (
        "shared-reference-once-insufficient",
        "fn need<F:FnOnce(u8)->u8>(){}fn f<F:FnOnce(u8)->u8>(){need::<&F>();}fn main(){}",
        false,
    ),
    (
        "mutable-reference-mutable",
        "fn need<F:FnMut(u8)->u8>(){}fn f<F:Fn(u8)->u8>(){need::<&mut F>();}fn main(){}\n",
        true,
    ),
    (
        "nested-shared-reference",
        "fn need<F:FnOnce(u8)->u8>(){}fn f<F:Fn(u8)->u8>(){need::<&&F>();}fn main(){}\n",
        true,
    ),
    (
        "nested-mutable-inside-shared",
        "fn need<F:FnOnce(u8)->u8>(){}fn f<F:Fn(u8)->u8>(){need::<&&mut F>();}fn main(){}\n",
        false,
    ),
    (
        "reference-caller-premise",
        "fn need<F:Fn()>(){}fn f<T:?Sized>()where for<'a>&'a T:Fn(){need::<&&T>();}fn main(){}\n",
        true,
    ),
    (
        "nested-callable-reference",
        "fn need<F:FnOnce(u8)->u8>(){}fn f(){need::<&&mut &fn(u8)->u8>();}fn main(){}\n",
        false,
    ),
];
#[test]
fn callable_implementations_preserve_signature_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("call-implementation-{name}"), source, *accepted)?;
        let result = checked(source);
        if result.is_ok() != *accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
fn checked(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn reference_forwarding_scales_through_aliases() -> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for depth in [8, 32, 96] {
        let mut source = String::from("fn need<F:Fn(u8)->u8>(){}type R0=fn(u8)->u8;");
        for level in 1..=depth {
            write!(source, "type R{level}=&'static R{};", level - 1)?;
        }
        write!(source, "fn main(){{need::<R{depth}>();}}")?;
        rustc::qualify(&format!("call-reference-depth-{depth}"), &source, true)?;
        checked(&source)?;
    }
    Ok(())
}

#[test]
fn reference_forwarding_infers_the_shared_function_item_argument()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for depth in [8, 32, 96] {
        let mut source =
            String::from("fn id<T>(x:T)->T{x}fn need<F:Fn(u8)->u8>(_:F){}fn main(){let f0=id;");
        for level in 1..=depth {
            write!(source, "let f{level}=&f{};", level - 1)?;
        }
        write!(source, "need(f{depth});}}")?;
        rustc::qualify(
            &format!("call-inferred-reference-depth-{depth}"),
            &source,
            true,
        )?;
        checked(&source)?;
    }
    Ok(())
}
