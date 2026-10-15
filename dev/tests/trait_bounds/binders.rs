//! Trait arguments use rigid alpha equality, independently qualified by rustc.
use super::cases;

#[test]
fn callable_binder_names_and_unused_slots_do_not_change_trait_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let rows = [
        (
            "reorder",
            "for<'x,'y>fn(&'x (),&'y ())->&'x ()",
            "for<'y,'x>fn(&'x (),&'y ())->&'x ()",
            true,
        ),
        ("unused", "for<'x>fn()->u8", "fn()->u8", true),
        (
            "extra-unused",
            "for<'x,'unused>fn(&'x ())->&'x ()",
            "for<'y>fn(&'y ())->&'y ()",
            true,
        ),
        (
            "independent-inputs",
            "for<'a,'b>fn(&'a (),&'b ())",
            "for<'a>fn(&'a (),&'a ())",
            false,
        ),
        (
            "independent-inputs-reverse",
            "for<'a>fn(&'a (),&'a ())",
            "for<'a,'b>fn(&'a (),&'b ())",
            false,
        ),
        (
            "output-correlation",
            "for<'a,'b>fn(&'a (),&'b ())->&'a ()",
            "for<'a,'b>fn(&'a (),&'b ())->&'b ()",
            false,
        ),
        (
            "local-static",
            "for<'a>fn(&'a ())",
            "fn(&'static ())",
            false,
        ),
        (
            "static-local",
            "fn(&'static ())",
            "for<'a>fn(&'a ())",
            false,
        ),
    ];
    for (name, given, needed, accepted) in rows {
        let source = format!(
            "trait A<U>{{}}fn g<T:A<{needed}>>(){{}}fn f<T:A<{given}>>(){{g::<T>()}}fn main(){{}}"
        );
        cases(&[(&format!("trait-binder-{name}"), &source, accepted)])?;
    }
    Ok(())
}

#[test]
fn local_binder_equations_cannot_be_proved_by_outer_lifetime_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "trait-binder-call-correlation",
            "trait A<U>{}fn g<T:A<for<'x,'y>fn(&'x (),&'y ())->&'y ()>>(t:T){}fn f<'a:'b,'b:'a,T:A<for<'x,'y>fn(&'x (),&'y ())->&'x ()>>(t:T){g(t)}fn main(){}",
            false,
        ),
        (
            "trait-binder-nominal-correlation",
            "trait A<U>{}struct S<T:A<for<'x,'y>fn(&'x (),&'y ())->&'y ()>>(T);fn f<'a:'b,'b:'a,T:A<for<'x,'y>fn(&'x (),&'y ())->&'x ()>>(t:T){let _:S<T>;}fn main(){}",
            false,
        ),
        (
            "trait-binder-outer-equality",
            "trait A<U>{}fn g<'b,T:A<fn(&'b ())>>(){}fn f<'a:'b,'b:'a,T:A<fn(&'a ())>>(){g::<'b,T>()}fn main(){}",
            true,
        ),
        (
            "trait-binder-outer-one-way",
            "trait A<U>{}fn g<'b,T:A<fn(&'b ())>>(){}fn f<'a:'b,'b,T:A<fn(&'a ())>>(){g::<'b,T>()}fn main(){}",
            false,
        ),
        (
            "trait-binder-captured-local",
            "trait A<U>{}fn g<'a,T:A<fn(&'a ())>>(){}fn f<'a,T:A<for<'x>fn(&'x ())>>(){g::<'a,T>()}fn main(){}",
            false,
        ),
        (
            "trait-binder-nominal-outer-equality",
            "trait A<U>{}struct S<'b,T:A<fn(&'b ())>>(T,&'b ());fn f<'a:'b,'b:'a,T:A<fn(&'a ())>>(t:T){let _:S<'b,T>;}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn inference_inside_callable_trait_arguments_cannot_capture_local_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "trait-binder-infer-byte",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U)>,U>(){}fn f<T:A<for<'x>fn(&'x (),u8)>>(){g::<T,_>()}fn main(){}",
            true,
        ),
        (
            "trait-binder-infer-outer",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U)>,U>(){}fn f<'a,T:A<for<'x>fn(&'x (),&'a ())>>(){g::<T,_>()}fn main(){}",
            true,
        ),
        (
            "trait-binder-prebound-outer",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U)>,U>(x:U){}fn f<'a,T:A<for<'x>fn(&'x (),&'a ())>>(x:&'a ()){g::<T,_>(x)}fn main(){}",
            true,
        ),
        (
            "trait-binder-repeated-inference",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U,U)>,U>(){}fn f<'a,T:A<for<'x>fn(&'x (),&'a (),&'a ())>>(){g::<T,_>()}fn main(){}",
            true,
        ),
        (
            "trait-binder-infer-local",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U)>,U>(){}fn f<T:A<for<'x>fn(&'x (),&'x ())>>(){g::<T,_>()}fn main(){}",
            false,
        ),
        (
            "trait-binder-infer-nested",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U)>,U>(){}fn f<T:A<for<'x>fn(&'x (),for<'y>fn(&'y ())->&'y ())>>(){g::<T,_>()}fn main(){}",
            true,
        ),
        (
            "trait-binder-infer-nested-capture",
            "trait A<U>{}fn g<T:A<for<'x>fn(&'x (),U)>,U>(){}fn f<T:A<for<'x>fn(&'x (),for<'y>fn(&'y (),&'x ()))>>(){g::<T,_>()}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn nested_signatures_and_nominal_arguments_preserve_binder_scope()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "trait-binder-nested-alpha",
            "trait A<U>{}fn g<T:A<for<'a>fn(&'a (),for<'b>fn(&'a (),&'b ())->&'a ())>>(){}fn f<T:A<for<'x>fn(&'x (),for<'y>fn(&'x (),&'y ())->&'x ())>>(){g::<T>()}fn main(){}",
            true,
        ),
        (
            "trait-binder-nested-depth",
            "trait A<U>{}fn g<T:A<for<'a>fn(&'a (),for<'b>fn(&'a (),&'b ())->&'b ())>>(){}fn f<T:A<for<'x>fn(&'x (),for<'y>fn(&'x (),&'y ())->&'x ())>>(){g::<T>()}fn main(){}",
            false,
        ),
        (
            "trait-binder-nominal-argument",
            "trait A<U>{}struct S<'a,'b>(&'a (),&'b ());fn g<T:A<for<'x>fn(S<'x,'x>)>>(){}fn f<T:A<for<'x,'y>fn(S<'x,'y>)>>(){g::<T>()}fn main(){}",
            false,
        ),
        (
            "trait-binder-different-assumptions",
            "trait A<U>{}fn g<T:A<for<'x,'y>fn(&'x (),&'y ())->&'y ()>>(){}fn f<T:A<for<'x,'y>fn(&'x (),&'y ())->&'x ()>+A<for<'x,'y>fn(&'x (),&'y ())->&'y ()>>(){g::<T>()}fn main(){}",
            true,
        ),
    ])
}
