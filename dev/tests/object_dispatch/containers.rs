//! Generic defaults use written owner bounds and preserve inferred parent lifetimes.
const CASES: &[(&str, &str, bool)] = &[
    (
        "container-inline",
        "trait A{}impl<'a>A for &'a u8{}struct H<'a,T:?Sized+'a>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){}",
        true,
    ),
    (
        "container-where",
        "trait A{}impl<'a>A for &'a u8{}struct H<'a,T:?Sized>where T:'a{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){}",
        true,
    ),
    (
        "container-alias",
        "trait A{}impl<'a>A for &'a u8{}type H<'a,T:'a> = &'a T;fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{x}fn main(){}",
        true,
    ),
    (
        "container-redundant-bound",
        "trait A{}impl<'a>A for &'a u8{}struct H<'a,T:?Sized+'a+'a>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){}",
        true,
    ),
    (
        "container-trait-argument",
        "trait A{}trait B<'a,T:?Sized+'a>{}fn f<'a>(x:&'a dyn B<'a,dyn A+'a>)->&'a dyn B<'a,dyn A>{x}fn main(){}",
        true,
    ),
    (
        "container-explicit-ambiguous",
        "trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}type P<'a,'b>=H<'a,'b,dyn A+'static>;fn main(){}",
        true,
    ),
    (
        "container-placeholder-ambiguous",
        "trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}fn f<'a>(x:&'a H<'a,'a,dyn A+'a>)->&'a H<'a,'a,dyn A+'_>{x}fn main(){}",
        true,
    ),
    (
        "container-static",
        "trait A{}struct H<T:?Sized+'static>{r:*const T}type P=H<dyn A>;fn main(){}",
        true,
    ),
    (
        "container-implied-field-not-default",
        "trait A{}impl<'a>A for &'a u8{}struct H<'a,T:?Sized>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){}",
        false,
    ),
    (
        "container-unbounded-alias",
        "trait A{}impl<'a>A for &'a u8{}type H<'a,T> = &'a T;fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{x}fn main(){}",
        false,
    ),
    (
        "container-ambiguous",
        "trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}type P<'a,'b>=H<'a,'b,dyn A>;fn main(){}",
        false,
    ),
    (
        "container-same-input-ambiguous",
        "trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}type P<'a>=H<'a,'a,dyn A>;fn main(){}",
        false,
    ),
    (
        "container-static-still-ambiguous",
        "trait A{}struct H<'a,T:?Sized+'a+'static>{r:&'a T}type P<'a>=H<'a,dyn A>;fn main(){}",
        false,
    ),
];

const RUNTIME: &[(&str, &str, bool)] = &[
    (
        "container-runtime",
        "trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,T:?Sized+'a>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){let v=7u8;let r=&v;let h=f(&r);check(h.r.a()==7);}",
        false,
    ),
    (
        "container-local-elision",
        "trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,T:?Sized+'a>{r:&'a T}fn main(){let v=7u8;let r=&v;let h:H<dyn A>=H{r:&r};check(h.r.a()==7);}",
        false,
    ),
    (
        "container-placeholder-input",
        "trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,T:?Sized+'a>{r:&'a T}fn main(){let v=7u8;let r=&v;let h:H<'_,dyn A>=H{r:&r};check(h.r.a()==7);}",
        false,
    ),
    (
        "container-partial-annotation",
        "trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,X,T:?Sized+'a>{x:X,r:&'a T}fn main(){let v=7u8;let r=&v;let h:H<'_,_,dyn A>=H{x:2u8,r:&r};check(h.r.a()==7&&h.x==2);}",
        false,
    ),
    (
        "container-output-elision",
        "trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,T:?Sized+'a>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'_,dyn A>{H{r:x}}fn main(){let v=7u8;let r=&v;let h=f(&r);check(h.r.a()==7);}",
        false,
    ),
];

#[test]
fn defaults_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    for &(name, source, accepted) in CASES {
        super::rustc::qualify(name, source, accepted)?;
        let result = super::build(source);
        if result.is_ok() != accepted {
            errors.push(format!(
                "{name}: expected {accepted}, {}",
                result.err().map_or("accepted".into(), |e| e.to_string())
            ));
        }
    }
    for &(name, source, _) in RUNTIME {
        let source = format!("{}{source}", super::native::CHECK);
        super::rustc::qualify(name, &source, true)?;
        if let Err(error) = super::build(&source) {
            errors.push(format!("{name}: {error}"));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn defaults_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("pa24-pa25-work/native-object-containers", RUNTIME)
}
