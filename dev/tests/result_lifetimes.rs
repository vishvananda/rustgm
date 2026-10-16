//! Result lifetime bounds preserve opaque identity and original formation obligations.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "copy-borrow-out",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn get<T:A>(x:&<T as A>::Out)-><T as A>::Out{*x}fn main(){check(get::<u8>(&73)==73);}\n",
        true,
    ),
    (
        "shared-forward",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn get<'a,T:A>(x:&'a<T as A>::Out)->&'a<T as A>::Out{x}fn main(){check(*get::<u8>(&73)==73);}\n",
        true,
    ),
    (
        "mutable-forward",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn get<'a,T:A>(x:&'a mut<T as A>::Out)->&'a mut<T as A>::Out{x}fn main(){let mut x=1;*get::<u8>(&mut x)=73;check(x==73);}\n",
        true,
    ),
    (
        "local-storage-escape",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn get<'a,T:A>(x:<T as A>::Out)->&'a<T as A>::Out{&x}fn main(){}\n",
        false,
    ),
    (
        "implied-result-bound",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn need<'a,X:'a>(){}fn get<'a,T:A>(_:&'a<T as A>::Out){need::<'a,<T as A>::Out>();}fn main(){get::<u8>(&73);}\n",
        true,
    ),
    (
        "implied-result-not-input",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn need<'a,X:'a>(){}fn get<'a,T:A>(_:&'a<T as A>::Out){need::<'a,T>();}fn main(){}\n",
        false,
    ),
    (
        "explicit-result-static",
        "trait A{type Out:Copy;}impl<'a>A for &'a u8{type Out=u16;}fn need<X:'static>(_:X){}fn get<T:A>(_:T,x:<T as A>::Out)where<T as A>::Out:'static{need(x);}fn main(){let n=1;get(&n,73);}\n",
        true,
    ),
    (
        "all-inputs-static",
        "trait A<U>{type Out:Copy;}impl A<u16>for u8{type Out=u32;}fn need<X:'static>(_:X){}fn get<T:A<U>+'static,U:'static>(x:<T as A<U>>::Out){need(x);}fn main(){get::<u8,u16>(73);}\n",
        true,
    ),
    (
        "missing-input-static",
        "trait A<U>{type Out:Copy;}fn need<X:'static>(_:X){}fn get<T:A<U>+'static,U>(x:<T as A<U>>::Out){need(x);}fn main(){}\n",
        false,
    ),
    (
        "result-static-not-input",
        "trait A{type Out:Copy;}fn need<X:'static>(_:X){}fn get<T:A>(x:T)where<T as A>::Out:'static{need(x);}fn main(){}\n",
        false,
    ),
    (
        "result-bound-not-other-member",
        "trait A{type Out:Copy;type Other:Copy;}fn need<X:'static>(_:X){}fn get<T:A>(x:<T as A>::Other)where<T as A>::Out:'static{need(x);}fn main(){}\n",
        false,
    ),
    (
        "implied-lifetime-argument-result",
        "trait A<'x>{type Out:Copy;}fn need<'a,X:'a>(){}fn get<'x,'a,T:A<'x>>(x:&'a<T as A<'x>>::Out){need::<'a,<T as A<'x>>::Out>();}fn main(){}\n",
        true,
    ),
    (
        "implied-lifetime-not-input",
        "trait A<'x>{type Out:Copy;}fn need<'a,X:'a>(){}fn get<'x,'a,T:A<'x>>(x:&'a<T as A<'x>>::Out){need::<'a,&'x u8>();}fn main(){}\n",
        false,
    ),
    (
        "distinct-application-assumption",
        "trait A<'a>{type Out:Copy;}fn need<'a,X:'a>(){}fn get<'a,'b,T:A<'a>+A<'b>>(x:&'a<T as A<'a>>::Out){need::<'a,<T as A<'b>>::Out>();}fn main(){}\n",
        false,
    ),
    (
        "copied-reference-outlives-storage",
        "trait A{type Out:Copy;}impl<'a>A for &'a u16{type Out=&'a u16;}fn get<T:A>(x:&<T as A>::Out)-><T as A>::Out{*x}fn main(){let x=73;let y;{let p=&x;y=get::<&u16>(&p);}check(*y==73);}\n",
        true,
    ),
    (
        "copied-reference-dangling",
        "trait A{type Out:Copy;}impl<'a>A for &'a u16{type Out=&'a u16;}fn get<T:A>(x:&<T as A>::Out)-><T as A>::Out{*x}fn main(){let y;{let x=73;let p=&x;y=get::<&u16>(&p);}check(*y==73);}\n",
        false,
    ),
    (
        "borrowed-result-erases-short-input",
        "trait A{type Out:Copy;}impl<'a>A for &'a u8{type Out=u16;}fn get<'a,T:A>(_:T,x:&'a<T as A>::Out)->&'a<T as A>::Out{x}fn main(){let x=73;let r;{let y=1;r=get(&y,&x);}check(*r==73);}\n",
        true,
    ),
];
#[test]
fn associated_result_lifetimes_do_not_grant_input_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("result-lifetime-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry = rustgm::sema::entry::select(&mut report, &facts)
                .map_err(|error| error.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_result_lifetimes_execute_with_independent_results()
-> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, accepted)| accepted.then_some((name, body, false)))
        .collect();
    native::check_cases("result-lifetimes", &cases)
}
