//! Inherited result abilities retain their original formation and lifetime debts.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "cyclic-with-copy",
        "trait C:D+Copy{}trait D:C{}trait A{type Out:C;}fn main(){}\n",
        false,
    ),
    (
        "cyclic-without-copy",
        "trait C:D{}trait D:C{}trait A{type Out:C;}fn main(){}\n",
        false,
    ),
    (
        "direct-copy",
        "trait C:Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "transitive-copy",
        "trait C:D{}trait D:Copy{}trait A{type Out:C;}impl D for u16{}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "diamond-copy",
        "trait C:L+R{}trait L:D{}trait R:D{}trait D:Copy{}trait A{type Out:C;}impl D for u16{}impl L for u16{}impl R for u16{}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "where-self-copy",
        "trait C where Self:Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "alias-self-copy",
        "type Id<T>=T;trait C where Id<Self>:Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "nested-alias-self-copy",
        "type Id<T>=T;type Alias<T>=Id<T>;trait C where Alias<Self>:Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "generic-supertrait-copy",
        "trait D<X>:Copy{}trait C:D<u8>{}trait A{type Out:C;}impl D<u8> for u16{}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "generic-bound-copy",
        "trait C<X>:Copy{}trait A{type Out:C<u8>;}impl C<u8> for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "forward-traits-copy",
        "trait C:D<u8>{}trait D<X>:Copy{}trait A{type Out:C;}impl D<u8> for u16{}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "relaxed-inherited-sized",
        "trait C:Copy{}trait A{type Out:?Sized+C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "late-trait-copy",
        "trait A{type Out:C;}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}trait C:Copy{}impl C for u16{}\n",
        true,
    ),
    (
        "clone-obligation",
        "trait C:Clone{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn need<X:Clone>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){f::<u8>();}\n",
        true,
    ),
    (
        "transitive-clone-obligation",
        "trait D:Clone{}trait C:D{}trait A{type Out:C;}impl D for u16{}impl C for u16{}impl A for u8{type Out=u16;}fn need<X:Clone>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){f::<u8>();}\n",
        true,
    ),
    (
        "stored-copy",
        "trait C:Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let p=dup::<u8>;let x=p(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "reference-result-copy",
        "trait C:Copy{}impl<'a>C for &'a u16{}trait A{type Out:C;}impl<'a>A for &'a u16{type Out=&'a u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let n=73;let x=dup::<&u16>(&n);check(*x.0==73&&*x.1==73);}\n",
        true,
    ),
    (
        "clone-does-not-copy",
        "trait C:Clone{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "unrelated-argument-copy",
        "trait C<X:Copy>{}trait A{type Out:C<u8>;}impl C<u8> for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "unrelated-result-copy",
        "trait C{type Other:Copy;}trait A{type Out:C;}impl C for u16{type Other=u8;}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "missing-result-impl",
        "trait C:Copy{}trait A{type Out:C;}impl A for u8{type Out=u16;}fn main(){}\n",
        false,
    ),
    (
        "invalid-copy-supertrait",
        "trait C:Copy{}struct D(u16);impl C for D{}trait A{type Out:C;}impl A for u8{type Out=D;}fn main(){}\n",
        false,
    ),
    (
        "unproved-parent",
        "trait C:Copy{}trait A{type Out:C;}fn bad<T>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){}\n",
        false,
    ),
    (
        "parenthesized-self-copy",
        "trait C where (Self):Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "alias-self-explicit-copy",
        "type Id<T>=T;trait C where Id<Self>:Copy{}trait A{type Out:C+Copy;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "inherited-sized-only",
        "trait C:Sized{}trait A{type Out:?Sized+C;}impl C for u16{}impl A for u8{type Out=u16;}fn need<X:Sized>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){f::<u8>();}\n",
        true,
    ),
    (
        "unrelated-argument-sized",
        "trait C<X:Sized>{}trait A{type Out:?Sized+C<u8>;}impl C<u8> for u16{}impl A for u8{type Out=u16;}fn need<X:Sized>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){f::<u8>();}\n",
        false,
    ),
    (
        "static-parent-proved",
        "trait C:Copy{}impl<'a:'static>C for &'a u16{}trait A{type Out:C;}impl<'a:'static>A for &'a u16{type Out=&'a u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static N:u16=73;fn main(){let x=dup::<&u16>(&N);check(*x.0==73&&*x.1==73);}\n",
        true,
    ),
    (
        "static-parent-local-rejected",
        "trait C:Copy{}impl<'a:'static>C for &'a u16{}trait A{type Out:C;}impl<'a:'static>A for &'a u16{type Out=&'a u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let n=73;let x=dup::<&u16>(&n);check(*x.0==73&&*x.1==73);}\n",
        false,
    ),
];

#[test]
fn inherited_result_abilities_require_complete_implementations()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("inherited-ability-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry = rustgm::sema::entry::select(&mut report, &facts)
                .map_err(|error| error.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        if name == "static-parent-local-rejected" {
            assert!(
                result
                    .err()
                    .ok_or("missing lifetime rejection")?
                    .contains("required lifetime does not outlive its target")
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn inherited_result_abilities_execute_with_independent_results()
-> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, accepted)| accepted.then_some((name, body, false)))
        .collect();
    native::check_cases("inherited-result-abilities", &cases)
}
