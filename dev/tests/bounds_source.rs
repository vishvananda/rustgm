use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "anonymous-bound-reference",
        "fn f<T>()where &T:Copy{}",
        false,
    ),
    (
        "placeholder-bound-reference",
        "fn f<T>()where &'_ T:Copy{}",
        false,
    ),
    (
        "nested-function-elision",
        "fn f<T>()where fn(&T):Copy{}",
        true,
    ),
    (
        "repeated-relaxation",
        "fn f<T:?Sized+?Sized>(x:&T){}",
        false,
    ),
    (
        "where-repeated-relaxation",
        "fn f<T:?Sized>(x:&T)where T:?Sized{}",
        false,
    ),
    ("relaxed-composite", "fn f<T>()where(T,):?Sized{}", false),
    (
        "relaxed-parenthesized",
        "fn f<T>(x:&T)where(T):?Sized{}",
        false,
    ),
    ("copy-trait-arguments", "fn f<T:Copy<u8>>(x:T){}", false),
    ("sized-trait-arguments", "fn f<T:Sized<u8>>(x:T){}", false),
    ("closed-false-alias", "type A<T> where [u8]:Copy=T;", true),
    (
        "closed-false-alias-use",
        "type A<T> where [u8]:Copy=T;fn f(x:A<u8>)->u8{x}",
        true,
    ),
    (
        "impossible-lifetime-forward",
        "fn f<'a>()where &'a mut u8:Copy{}fn g<'b>()where &'b mut u8:Copy{f::<'b>();}",
        true,
    ),
    ("closed-false-nongeneric", "fn f()where str:Sized{}", false),
    (
        "closed-true-nongeneric",
        "fn f()where u8:Copy{}fn run(){f();}",
        true,
    ),
    (
        "erased-clone-item",
        "struct N;fn f<T:Clone>(){}fn run(){let x=f::<N>;}",
        false,
    ),
    (
        "clone-does-not-leak",
        "fn f<T:Clone>(x:T)->T{x}fn g<U>(x:U)->U{f(x)}",
        false,
    ),
    (
        "copy-does-not-leak",
        "fn f<T:Copy>(x:T)->[T;2]{[x;2]}fn g<U>(x:U)->[U;2]{[x;2]}",
        false,
    ),
    (
        "scoped-array-well-formed",
        "fn f<T:?Sized+Copy>(x:[T;2])->[[T;2];2]{[x;2]}",
        true,
    ),
    (
        "ill-formed-array-assumption",
        "fn f<T:?Sized>()where[T;0]:Sized{}",
        false,
    ),
    (
        "ill-formed-tuple-assumption",
        "fn f<T:?Sized>()where(T,u8):Sized{}",
        false,
    ),
    (
        "tuple-sized-assumption",
        "fn f<T:?Sized>(x:(T,))->(T,)where(T,):Sized{x}",
        true,
    ),
    (
        "alias-bound-not-proof",
        "type A<T:Copy>=T;fn f<U>(x:A<U>)->[U;2]{[x;2]}",
        false,
    ),
    (
        "forward-core-alias",
        "use c::marker::Copy as C;use core as c;fn f<T:C>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "forward-module-alias",
        "use m::Copy as C;use core::marker as m;fn f<T:C>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "reexport-copy",
        "mod m{pub use core::marker::Copy as C;}fn f<T:m::C>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "explicit-import-shadow",
        "mod m{pub trait C{}}use m::C as Copy;fn f<T:Copy>(x:T)->[T;2]{[x;2]}",
        false,
    ),
    (
        "nested-no-prelude",
        "#[no_implicit_prelude]mod m{pub fn f<T:Copy>(x:T){}}",
        false,
    ),
    (
        "lifetime-bound-shared",
        "fn f<'a>()where &'a u8:Copy{}fn run(){f();}",
        true,
    ),
    (
        "lifetime-bound-mut",
        "fn f<'a>()where &'a mut u8:Copy{}fn run(){f();}",
        false,
    ),
    (
        "lifetime-impossible-unused",
        "fn f<'a>()where &'a mut u8:Copy{}",
        true,
    ),
    (
        "copy-repeat",
        "fn f<T:Copy>(x:T)->[T;2]{[x;2]}fn run()->[u8;2]{f(3)}",
        true,
    ),
    (
        "clone-identity",
        "fn f<T:Clone>(x:T)->T{x}fn run()->u8{f(3)}",
        true,
    ),
    ("clone-repeat", "fn f<T:Clone>(x:T)->[T;2]{[x;2]}", false),
    (
        "sized-explicit",
        "fn f<T:Sized>(x:T)->T{x}fn run()->u8{f(3)}",
        true,
    ),
    (
        "relaxed-sized",
        "fn f<T:?Sized>(x:&T)->&T{x}fn run(x:&str)->&str{f(x)}",
        true,
    ),
    ("relaxed-value", "fn f<T:?Sized>(x:T){}", false),
    (
        "copy-relaxed",
        "fn f<T:?Sized+Copy>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    ("clone-relaxed", "fn f<T:?Sized+Clone>(x:T)->T{x}", true),
    ("where-copy", "fn f<T>(x:T)->[T;2]where T:Copy{[x;2]}", true),
    ("where-clone", "fn f<T>(x:T)->T where T:Clone{x}", true),
    (
        "where-relaxed",
        "fn f<T>(x:&T)->&T where T:?Sized{x}fn run(x:&str)->&str{f(x)}",
        true,
    ),
    (
        "where-parentheses",
        "fn f<T>(x:T)->[T;2]where (T):Copy{[x;2]}",
        true,
    ),
    (
        "tuple-assumption",
        "fn f<T,U>(x:(T,U))->[(T,U);2]where(T,U):Copy{[x;2]}",
        true,
    ),
    (
        "tuple-reverse-inference",
        "fn f<T,U>(x:T)->[T;2]where(T,U):Copy{[x;2]}",
        false,
    ),
    ("trivial-copy", "fn f<T>(x:T)->T where u8:Copy{x}", true),
    (
        "trivial-copy-false",
        "fn f<T>(x:T)->T where [u8]:Copy{x}",
        false,
    ),
    ("special-slice-unused", "fn f<T>()where[T]:Copy{}", true),
    (
        "special-slice-called",
        "fn f<T>()where[T]:Copy{}fn run(){f::<u8>();}",
        false,
    ),
    (
        "special-mut-unused",
        "fn f<'a,T>()where &'a mut T:Copy{}",
        false,
    ),
    (
        "special-mut-called",
        "fn f<'a,T>()where &'a mut T:Copy{}fn run(){f::<u8>();}",
        false,
    ),
    (
        "copy-missing-use",
        "struct N(u8);fn f<T:Copy>(x:T)->T{x}fn run(x:N)->N{f(x)}",
        false,
    ),
    (
        "clone-missing-use",
        "struct N(u8);fn f<T:Clone>(x:T)->T{x}fn run(x:N)->N{f(x)}",
        false,
    ),
    (
        "copy-forward",
        "fn g<T:Copy>(x:T)->T{x}fn f<U:Copy>(x:U)->U{g(x)}",
        true,
    ),
    (
        "copy-forward-missing",
        "fn g<T:Copy>(x:T)->T{x}fn f<U>(x:U)->U{g(x)}",
        false,
    ),
    (
        "qualified-copy",
        "fn f<T:core::marker::Copy>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "qualified-clone",
        "fn f<T:core::clone::Clone>(x:T)->T{x}",
        true,
    ),
    (
        "aliased-copy",
        "use core::marker::Copy as C;fn f<T:C>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "no-std-copy",
        "#![no_std]\nfn f<T:Copy>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "no-prelude-copy",
        "#![no_implicit_prelude]\nfn f<T:Copy>(x:T)->[T;2]{[x;2]}",
        false,
    ),
    (
        "explicit-core-no-prelude",
        "#![no_implicit_prelude]\nextern crate core;fn f<T:core::marker::Copy>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "shadow-copy-struct",
        "struct Copy;fn f<T:Copy>(x:T)->[T;2]{[x;2]}",
        false,
    ),
    (
        "shadow-copy-trait",
        "trait Copy{}fn f<T:Copy>(x:T)->[T;2]{[x;2]}",
        false,
    ),
    ("optional-clone", "fn f<T:?Clone>(x:T)->T{x}", false),
    (
        "alias-unenforced",
        "type A<T:Copy>=T;struct N;fn f(x:A<N>)->N{x}",
        true,
    ),
    (
        "alias-relaxed",
        "type A<T:?Sized>=T;fn f(x:&A<str>)->&str{x}",
        true,
    ),
    (
        "copy-slice-reference",
        "fn f<T:Copy>(x:T)->[T;2]{[x;2]}fn run(x:&[u8])->[&[u8];2]{f(x)}",
        true,
    ),
    (
        "copy-mut-reference",
        "fn f<T:Copy>(x:T)->[T;2]{[x;2]}fn run(x:&mut u8){f(x);}",
        false,
    ),
    (
        "copy-derived",
        "#[derive(Copy,Clone)]struct N(u8);fn f<T:Copy>(x:T)->[T;2]{[x;2]}fn run(x:N)->[N;2]{f(x)}",
        true,
    ),
    (
        "impossible-slice-forward",
        "fn f<T>()where[T]:Copy{}fn g<U>()where[U]:Copy{f::<U>();}",
        true,
    ),
    (
        "array-assumption",
        "fn f<T,const N:usize>(x:[T;N])->[[T;N];2]where[T;N]:Copy{[x;2]}",
        true,
    ),
    (
        "array-reverse-inference",
        "fn f<T,const N:usize>(x:T)->[T;2]where[T;N]:Copy{[x;2]}",
        false,
    ),
    (
        "tuple-copy-implies-clone",
        "fn cloneable<T:Clone>(x:T)->T{x}fn f<T,U>(x:(T,U))->(T,U)where(T,U):Copy{cloneable(x)}",
        true,
    ),
    (
        "tuple-clone-not-copy",
        "fn f<T,U>(x:(T,U))->[(T,U);2]where(T,U):Clone{[x;2]}",
        false,
    ),
    (
        "unused-bounded-item",
        "struct N;fn f<T:Copy>(x:T)->T{x}fn run(){let x=f::<N>;}",
        false,
    ),
    (
        "erased-bounded-item",
        "struct N;fn f<T:Copy>(){}fn run(){let x=f::<N>;}",
        false,
    ),
    (
        "core-alias",
        "use core as c;use c::marker::Copy as C;fn f<T:C>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "module-alias",
        "use core::marker as m;use m::Copy as C;fn f<T:C>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "marker-glob",
        "use core::marker::*;fn f<T:Copy+Sized>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "root-core-shadow",
        "mod core{pub mod marker{pub trait Copy{}}}fn f<T:core::marker::Copy>(x:T)->[T;2]{[x;2]}",
        false,
    ),
    (
        "absolute-core-shadow",
        "mod core{pub mod marker{pub trait Copy{}}}fn f<T: ::core::marker::Copy>(x:T)->[T;2]{[x;2]}",
        true,
    ),
    (
        "where-copy-relaxed",
        "fn f<T:?Sized>(x:T)->[T;2]where T:Copy{[x;2]}",
        true,
    ),
    (
        "unbounded-generic-use",
        "fn f<T:?Sized>(x:&T)->&T{x}fn g<U:?Sized>(x:&U)->&U{f(x)}",
        true,
    ),
];

#[test]
fn builtin_bounds_use_scoped_assumptions_and_check_instantiations() -> Result<(), String> {
    let errors = CASES
        .iter()
        .filter_map(|(name, source, accepted)| {
            let result = semantic::analyze(source).and_then(|(_, facts)| {
                let contracts = rustgm::body::regions::collect(&facts).map_err(str::to_owned)?;
                rustgm::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
            (result.is_ok() != *accepted).then(|| {
                format!(
                    "{name}: expected {accepted}: {}",
                    result.err().unwrap_or_default()
                )
            })
        })
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
#[test]
fn builtin_bound_expectations_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("builtin-bound-{name}"), source, *accepted)?;
    }
    Ok(())
}
