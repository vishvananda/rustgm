//! Auto-trait evidence can use hidden results while object methods keep their contracts.
const CASES: &[(&str, &str, bool)] = &[
    (
        "hidden-constant-pattern",
        "fn main(){let x=make();need(&x);check(x.f()==7);}fn need<T:Send>(_:&T){}trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}const C:(u8,u8)=(1,2);fn make()->impl A{match (1u8,2u8){C=>7u8,_=>9}}",
        false,
    ),
    (
        "hidden-final-return-marker",
        "fn main(){let x=outer();let p:&(dyn A+Send)=&x;check(p.f()==7);}trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn outer()->impl A+Send{inner()}fn inner()->impl A{7u8}",
        false,
    ),
    (
        "hidden-final-nominal",
        "fn main(){let w=W(make());let p:&(dyn A+Send)=&w.0;check(p.f()==9);}struct W<T:Send>(T);trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn make()->impl A{9u8}",
        false,
    ),
    (
        "hidden-final-quantified",
        "fn main(){let x=outer();check(x.f(&1)==12);}trait A{}impl A for u8{}trait B<'a>{fn f(&self,x:&'a u8)->u8;}impl<'a,T:Send>B<'a> for T{fn f(&self,x:&'a u8)->u8{*x+11}}fn outer()->impl for<'a>B<'a>{inner()}fn inner()->impl A{7u8}",
        false,
    ),
    (
        "hidden-marker-suspended-borrow",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn main(){let x=make0();check(x.f()==7);}fn need<T:Send>(_:T){}fn make0()->impl A{let n=7u8;let p=&n;need(make1());*p}fn make1()->impl A{let n=9u8;let p=&n;*p}",
        false,
    ),
    (
        "hidden-marker-object",
        "fn main(){let x=make();let p:&(dyn A+Send)=&x;check(p.f()==7);}trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn make()->impl A{7u8}",
        false,
    ),
    (
        "hidden-marker-generic",
        "trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn make()->impl A{9u8}fn pass<T:Send>(x:T)->T{x}fn main(){let x=pass(make());check(x.f()==9);}",
        false,
    ),
    (
        "hidden-marker-manual",
        "trait A{fn f(&self)->u8;}struct W(*const u8,u8);unsafe impl Send for W{}impl A for W{fn f(&self)->u8{self.1}}fn make(x:*const u8)->impl A{W(x,11)}fn main(){let n=3u8;let x=make(&n as *const u8);let p:&(dyn A+Send)=&x;check(p.f()==11);}",
        false,
    ),
];

#[test]
fn hidden_marker_dispatch_lowers() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in CASES {
        let source = format!("{}{source}", super::native::CHECK);
        super::rustc::qualify(name, &source, true)?;
        super::build(&source).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn hidden_marker_dispatch_executes() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("pa24-pa25-work/native-hidden-markers", CASES)
}
