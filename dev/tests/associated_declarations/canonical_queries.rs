//! Canonical premise reuse preserves complete formation, equality and lifetime gates.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "recursive-lifetimes",
        "trait A<'a>{type Out:A<'a>+Copy;}impl<'a>A<'a>for u8{type Out=u8;}fn f<'a,T:A<'a>>(x:<<<T as A<'a>>::Out as A<'a>>::Out as A<'a>>::Out)->(<<<T as A<'a>>::Out as A<'a>>::Out as A<'a>>::Out,<<<T as A<'a>>::Out as A<'a>>::Out as A<'a>>::Out){(x,x)}fn main(){let(a,b)=f::<u8>(7);check(a==7&&b==7);}",
        true,
    ),
    (
        "projected-caller-premise",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}fn f<'a,T:A<'a>>(x:<<T as A<'a>>::Out as A<'a>>::Out)-><<T as A<'a>>::Out as A<'a>>::Out where <T as A<'a>>::Out:A<'a>{x}fn main(){check(f::<u8>(7)==7);}",
        true,
    ),
    (
        "result-equality",
        "trait A<'a>{type Out:A<'a,Out=u8>;}impl<'a>A<'a>for u8{type Out=u8;}fn f<'a,T:A<'a>>(x:<<T as A<'a>>::Out as A<'a>>::Out)->u8{x}fn main(){check(f::<u8>(7)==7);}",
        true,
    ),
    (
        "caller-equality",
        "trait A<'a>{type Out:A<'a>;}impl<'a>A<'a>for u8{type Out=u8;}fn f<'a,T:A<'a>>(x:<<T as A<'a>>::Out as A<'a>>::Out)->u8 where <T as A<'a>>::Out:A<'a,Out=u8>{x}fn main(){check(f::<u8>(7)==7);}",
        true,
    ),
    (
        "erased-input-lifetime",
        "trait A<'a>{type Out:A<'a>;}impl<'a>A<'a>for u8{type Out=u8;}trait P<'a>{type Value:A<'a>;}impl<'a,'b:'a,T:A<'a>>P<'a>for &'b T{type Value=T;}fn f<'a,'b:'a,T:A<'a>+'b>(x:<<<&'b T as P<'a>>::Value as A<'a>>::Out as A<'a>>::Out)-><<<&'b T as P<'a>>::Value as A<'a>>::Out as A<'a>>::Out{x}fn main(){check(f::<u8>(7)==7);}",
        true,
    ),
    (
        "missing-erased-input-lifetime",
        "trait A<'a>{type Out:A<'a>;}impl<'a>A<'a>for u8{type Out=u8;}trait P<'a>{type Value:A<'a>;}impl<'a,'b:'a,T:A<'a>>P<'a>for &'b T{type Value=T;}fn f<'a,'b,T:A<'a>+'b>(x:<<<&'b T as P<'a>>::Value as A<'a>>::Out as A<'a>>::Out)-><<<&'b T as P<'a>>::Value as A<'a>>::Out as A<'a>>::Out{x}fn main(){}",
        false,
    ),
    (
        "reversed-erased-input-lifetime",
        "trait A<'a>{type Out:A<'a>;}impl<'a>A<'a>for u8{type Out=u8;}trait P<'a>{type Value:A<'a>;}impl<'a,'b:'a,T:A<'a>>P<'a>for &'b T{type Value=T;}fn f<'a:'b,'b,T:A<'a>+'b>(x:<<<&'b T as P<'a>>::Value as A<'a>>::Out as A<'a>>::Out)-><<<&'b T as P<'a>>::Value as A<'a>>::Out as A<'a>>::Out{x}fn main(){}",
        false,
    ),
    (
        "different-result-lifetime",
        "trait A<'a>{type Out:A<'a>;}fn f<'a,'b,T:A<'a>>(x:<<T as A<'a>>::Out as A<'b>>::Out)-><<T as A<'a>>::Out as A<'b>>::Out{x}fn main(){}",
        false,
    ),
    (
        "deferred-alias",
        "trait A<'a>{type Out:A<'a>;}impl<'a>A<'a>for u8{type Out=u8;}type Value<'a,T>=<<T as A<'a>>::Out as A<'a>>::Out;fn f<'a,T:A<'a>>(x:Value<'a,T>)->Value<'a,T>{x}fn main(){check(f::<u8>(7)==7);}",
        true,
    ),
];

#[test]
fn canonical_projection_premises_match_independent_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("canonical-premise-{name}"), &source, accepted)?;
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
fn canonical_projection_premises_preserve_native_values() -> Result<(), Box<dyn std::error::Error>>
{
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("canonical-projection-premises", &cases)
}
