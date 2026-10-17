//! Trait header names are prepared separately from complete result contracts.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "own-shorthand",
        "trait A where Self::Out:Copy{type Out;fn twice(x:Self::Out)->(Self::Out,Self::Out){(x,x)}}impl A for u8{type Out=u16;}fn main(){let pair=<u8 as A>::twice(7);check(pair.0==7&&pair.1==7);}",
        true,
    ),
    (
        "own-qualified",
        "trait A where <Self as A>::Out:Copy{type Out;fn twice(x:Self::Out)->(Self::Out,Self::Out){(x,x)}}impl A for u8{type Out=u16;}fn main(){let pair=<u8 as A>::twice(7);check(pair.0==7&&pair.1==7);}",
        true,
    ),
    (
        "own-unused-invalid",
        "trait A where Self::Out:Copy{type Out;}impl A for u8{type Out=&'static mut u8;}fn main(){}",
        false,
    ),
    (
        "inherited-forward",
        "trait A:B where Self::Out:Copy{fn twice(x:Self::Out)->(Self::Out,Self::Out){(x,x)}}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u8{}fn main(){let pair=<u8 as A>::twice(7);check(pair.0==7&&pair.1==7);}",
        true,
    ),
    (
        "inherited-reversed",
        "trait B{type Out;}trait A:B where Self::Out:Copy{fn twice(x:Self::Out)->(Self::Out,Self::Out){(x,x)}}impl B for u8{type Out=u16;}impl A for u8{}fn main(){let pair=<u8 as A>::twice(7);check(pair.0==7&&pair.1==7);}",
        true,
    ),
    (
        "inherited-unused-invalid",
        "trait A:B where Self::Out:Copy{}trait B{type Out;}impl B for u8{type Out=&'static mut u8;}impl A for u8{}fn main(){}",
        false,
    ),
    (
        "inherited-ambiguous",
        "trait A:B+C where Self::Out:Copy{}trait B{type Out;}trait C{type Out;}fn main(){}",
        false,
    ),
    (
        "inherited-qualified",
        "trait A:B+C where <Self as B>::Out:Copy{}trait B{type Out;}trait C{type Out;}impl B for u8{type Out=u16;}impl C for u8{type Out=&'static mut u8;}impl A for u8{}fn main(){}",
        true,
    ),
    (
        "parameter-bound",
        "trait A<T:B> where T::Out:Copy{}trait B{type Out;}impl B for u8{type Out=u16;}impl A<u8>for u16{}fn main(){}",
        true,
    ),
    (
        "parameter-bound-reordered",
        "trait A<T> where T::Out:Copy,T:B{}trait B{type Out;}impl B for u8{type Out=u16;}impl A<u8>for u16{}fn main(){}",
        true,
    ),
    (
        "parameter-bound-invalid",
        "trait A<T:B> where T::Out:Copy{}trait B{type Out;}impl B for u8{type Out=&'static mut u8;}impl A<u8>for u16{}fn main(){}",
        false,
    ),
    (
        "inherited-lifetime",
        "trait A<'a>:B<'a> where Self::Out:Copy{fn twice(x:Self::Out)->(Self::Out,Self::Out){(x,x)}}trait B<'a>{type Out;}impl<'a>B<'a>for u8{type Out=&'a u8;}impl<'a>A<'a>for u8{}fn main(){let x=7u8;let pair=<u8 as A<'_>>::twice(&x);check(*pair.0==7&&*pair.1==7);}",
        true,
    ),
    (
        "inherited-mixed-arguments",
        "trait A<T:Copy,const N:usize>:B<T,N> where Self::Out:Copy{}trait B<T:Copy,const N:usize>{type Out;}impl<T:Copy,const N:usize>B<T,N>for u8{type Out=[T;N];}impl<T:Copy,const N:usize>A<T,N>for u8{}fn main(){}",
        true,
    ),
    (
        "result-user-trait",
        "trait A where Self::Out:Need{type Out;fn use_it(x:Self::Out)->u8{<Self::Out as Need>::value(x)}}trait Need:Sized{fn value(self)->u8;}impl Need for u16{fn value(self)->u8{self as u8}}impl A for u8{type Out=u16;}fn main(){check(<u8 as A>::use_it(7)==7);}",
        true,
    ),
    (
        "supertrait-cycle",
        "trait A:B where Self::Out:Copy{type Out;}trait B:A{}fn main(){}",
        false,
    ),
    (
        "missing-parent-proof",
        "trait A where <Self as B>::Out:Copy{}trait B{type Out;}fn main(){}",
        false,
    ),
    (
        "owned-result-implied",
        "trait A where Self::Out:Copy{type Out;}impl A for u8{type Out=u16;}fn twice<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=twice::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "inherited-result-explicit",
        "trait A:B where Self::Out:Copy{}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u8{}fn twice<T:A>(x:T::Out)->(T::Out,T::Out)where T::Out:Copy{(x,x)}fn main(){let p=twice::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "result-user-trait-implied",
        "trait A where Self::Out:Need{type Out;}trait Need:Sized{fn value(self)->u8;}impl Need for u16{fn value(self)->u8{self as u8}}impl A for u8{type Out=u16;}fn use_it<T:A>(x:T::Out)->u8{<T::Out as Need>::value(x)}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "result-user-trait-invalid",
        "trait A where Self::Out:Need{type Out;}trait Need{}impl A for u8{type Out=u16;}fn main(){}",
        false,
    ),
    (
        "inherited-implied",
        "trait A:B where Self::Out:Copy{}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u8{}fn twice<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=twice::<u8>(7);check(p.0==7&&p.1==7);}",
        false,
    ),
    (
        "own-super-argument",
        "trait A:B<Self::Out>{type Out;}trait B<T>{}impl B<u16>for u8{}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "own-qualified-super-argument",
        "trait A:B<<Self as A>::Out>{type Out;}trait B<T>{}impl B<u16>for u8{}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "projected-argument-cycle",
        "trait A:B<Self::Out>{type Out;}trait B<T>:A{}fn main(){}",
        false,
    ),
    (
        "concrete-header",
        "trait A where <u8 as B>::Out:Copy{}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u16{}fn main(){}",
        true,
    ),
    (
        "concrete-header-invalid",
        "trait A where <u8 as B>::Out:Copy{}trait B{type Out;}impl B for u8{type Out=&'static mut u8;}fn main(){}",
        false,
    ),
    (
        "result-alias",
        "type Out<T:?Sized+A>=<T as A>::Out;trait A where Out<Self>:Copy{type Out;}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "owned-builtin-through-trait",
        "trait A where Self::Out:Need{type Out;}trait Need:Copy{}impl Need for u16{}impl A for u8{type Out=u16;}fn twice<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=twice::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-result-default",
        "trait A{type Out:Need;fn use_it(x:Self::Out)->u8{<Self::Out as Need>::value(x)}}trait Need:Sized{fn value(self)->u8;}impl Need for u16{fn value(self)->u8{self as u8}}impl A for u8{type Out=u16;}fn main(){check(<u8 as A>::use_it(7)==7);}",
        true,
    ),
    (
        "member-result-generic",
        "trait A{type Out:Need;}trait Need:Sized{fn value(self)->u8;}impl Need for u16{fn value(self)->u8{self as u8}}impl A for u8{type Out=u16;}fn use_it<T:A>(x:T::Out)->u8{<T::Out as Need>::value(x)}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "captured-result-generic",
        "trait A<'a>where Self::Out:Need{type Out;}trait Need:Sized{fn value(self)->u8;}impl<'a>Need for &'a u8{fn value(self)->u8{*self}}impl<'a>A<'a>for u8{type Out=&'a u8;}fn use_it<'a,T:A<'a>>(x:T::Out)->u8{<T::Out as Need>::value(x)}fn main(){let x=7u8;check(use_it::<u8>(&x)==7);}",
        true,
    ),
    (
        "captured-result-default",
        "trait A<'a>where Self::Out:Need{type Out;fn use_it(x:Self::Out)->u8{<Self::Out as Need>::value(x)}}trait Need:Sized{fn value(self)->u8;}impl<'a>Need for &'a u8{fn value(self)->u8{*self}}impl<'a>A<'a>for u8{type Out=&'a u8;}fn main(){let x=7u8;check(<u8 as A<'_>>::use_it(&x)==7);}",
        true,
    ),
    (
        "unique-result-generic",
        "trait A<'a>where Self::Out:Need{type Out;}trait Need:Sized{fn value(self)->u8;}impl<'a>Need for &'a mut u8{fn value(self)->u8{*self+=1;*self}}impl<'a>A<'a>for u8{type Out=&'a mut u8;}fn use_it<'a,T:A<'a>>(x:T::Out)->u8{<T::Out as Need>::value(x)}fn main(){let mut x=7u8;check(use_it::<u8>(&mut x)==8);check(x==8);}",
        true,
    ),
    (
        "unique-result-reused",
        "trait A<'a>where Self::Out:Need{type Out;}trait Need:Sized{fn value(self)->u8;}fn use_it<'a,T:A<'a>>(x:T::Out)->u8{let _=<T::Out as Need>::value(x);<T::Out as Need>::value(x)}fn main(){}",
        false,
    ),
    (
        "optional-sized-copy",
        "trait A where Self::Out:Copy{type Out:?Sized;}impl A for u8{type Out=u16;}fn take<T:A>(x:T::Out)->T::Out{let _y=x;x}fn main(){check(take::<u8>(7)==7);}",
        true,
    ),
];

#[test]
fn trait_header_projections_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("trait-header-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn trait_header_projections_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("trait-header-projections", &cases)
}
