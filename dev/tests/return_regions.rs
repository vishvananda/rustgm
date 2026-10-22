//! Hidden references require capture, borrow and execution proofs in their defining body.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const VALUE: &str =
    "trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}";
const ACCEPTED: &[(&str, &str)] = &[
    (
        "input-reference",
        "fn f<'a>(x:&'a u8)->impl Value{x}fn main(){let x=7;check(f(&x).value()==7);}",
    ),
    (
        "elided-reference",
        "fn f(x:&u8)->impl Value{x}fn main(){let x=11;check(f(&x).value()==11);}",
    ),
    (
        "reborrow",
        "fn f<'a>(x:&'a u8)->impl Value{&*x}fn main(){let x=13;check(f(&x).value()==13);}",
    ),
    (
        "ordered-branches",
        "fn f<'a:'b,'b>(x:&'a u8,y:&'b u8,c:bool)->impl Value{if c{x}else{y}}fn main(){let x=7;let y=11;check(f(&x,&y,true).value()==7);check(f(&x,&y,false).value()==11);}",
    ),
    (
        "precise-input",
        "fn f<'a>(x:&'a u8)->impl Value+use<'a>{x}fn main(){let x=17;check(f(&x).value()==17);}",
    ),
    (
        "promoted-two-captures",
        "fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Value{&19u8}fn main(){let x=1;let y=2;check(f(&x,&y).value()==19);}",
    ),
    (
        "precise-promoted-escape",
        "fn f<'a>(_:&'a u8)->impl Value+use<>{&23u8}fn main(){let value;{let local=1;value=f(&local);}check(value.value()==23);}",
    ),
    (
        "nested-results",
        "fn f<'a,'b>(x:&'a u8,y:&'b u8)->(impl Value,impl Value){(x,y)}fn main(){let x=29;let y=31;let pair=f(&x,&y);check(pair.0.value()==29);check(pair.1.value()==31);}",
    ),
    (
        "shortened-capture",
        "fn f<'a,'b:'a>(_:&'a u8,x:&'b u8)->impl Value+use<'a>{x}fn main(){let x=37;let y=1;check(f(&y,&x).value()==37);}",
    ),
    (
        "equivalent-capture-names",
        "fn f<'a:'b,'b:'a>(x:&'a u8)->(impl Value+use<'a>,impl Value+use<'b>){(x,x)}fn main(){let x=41;let pair=f(&x);check(pair.0.value()==41);check(pair.1.value()==41);}",
    ),
    (
        "pointer-binder",
        "fn sink(_: &u8){}fn f()->impl Copy+use<>{sink as fn(&u8)}fn main(){let value=f();let _=(value,value);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "unrelated-branches",
        "fn f<'a,'b>(x:&'a u8,y:&'b u8,c:bool)->impl Value{if c{x}else{y}}fn main(){}",
    ),
    (
        "local-reference",
        "fn f()->impl Value{let x=7;&x}fn main(){}",
    ),
    (
        "excluded-input",
        "fn f<'a>(x:&'a u8)->impl Value+use<>{x}fn main(){}",
    ),
    (
        "unrelated-capture",
        "fn f<'a,'b>(_:&'a u8,x:&'b u8)->impl Value+use<'a>{x}fn main(){}",
    ),
    (
        "caller-escape",
        "fn f<'a>(x:&'a u8)->impl Value{x}fn main(){let value;{let local=7;value=f(&local);}let _=value.value();}",
    ),
    (
        "invariant-branches",
        "fn f<'a:'b,'b>(x:&'a mut &'a u8,y:&'b mut &'b u8,c:bool)->impl Sized{if c{x}else{y}}fn main(){}",
    ),
];

#[test]
fn references_match_independent_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, body) in cases {
            let source = format!("{}{VALUE}{body}", native::CHECK);
            rustc::qualify(&format!("return-regions-{name}"), &source, accepted)?;
            let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
                let entry =
                    rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
                rustgm::lower::instances::build(&report, &mut facts, entry)
                    .map_err(|e| e.to_string())
            });
            assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn references_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let sources: Vec<_> = ACCEPTED
        .iter()
        .map(|(name, body)| (*name, format!("{VALUE}{body}")))
        .collect();
    let cases: Vec<_> = sources
        .iter()
        .map(|(name, body)| (*name, body.as_str(), false))
        .collect();
    native::check_cases("opaque-reference-results/native", &cases)
}
