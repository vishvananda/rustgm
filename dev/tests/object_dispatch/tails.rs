//! Object-tail projections must align each nested prefix using vtable metadata.
const CASES: &[(&str, &str, bool)] = &[
    (
        "tail-shared",
        "trait A{fn a(&self)->u16;}impl A for u16{fn a(&self)->u16{*self}}#[repr(C)]struct W<T:?Sized>{tag:u8,value:T}fn read(x:&W<dyn A>)->u16{x.value.a()+x.tag as u16}fn main(){let w=W{tag:2,value:1000u16};check(read(&w)==1002);}",
        false,
    ),
    (
        "tail-two-alignments",
        "trait A{fn a(&self)->u16;}impl A for u8{fn a(&self)->u16{*self as u16}}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<dyn A>)->u16{x.1.a()+x.0 as u16}fn main(){let a=W(2,7u8);let b=W(3,1000u64);check(read(&a)==9&&read(&b)==1003);}",
        false,
    ),
    (
        "tail-nested",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<W<dyn A>>)->u16{x.1.1.a()+x.0 as u16+x.1.0 as u16}fn main(){let w=W(2,W(3,1000u64));check(read(&w)==1005);}",
        false,
    ),
    (
        "tail-nested-prefix-alignment",
        "trait A{fn a(&self)->u16;}impl A for u8{fn a(&self)->u16{*self as u16}}#[repr(C)]struct I<T:?Sized>(u64,T);#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<I<dyn A>>)->u16{x.1.1.a()+x.0 as u16+x.1.0 as u16}fn main(){let w=W(2,I(1000,7u8));check(read(&w)==1009);}",
        false,
    ),
    (
        "tail-mutable",
        "trait A{fn a(&mut self);}impl A for u64{fn a(&mut self){*self+=2;}}#[repr(C)]struct W<T:?Sized>(u8,T);fn change(x:&mut W<dyn A>){x.1.a();x.0+=3;}fn main(){let mut w=W(2,1000u64);change(&mut w);check(w.0==5&&w.1==1002);}",
        false,
    ),
    (
        "tail-reborrow",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<dyn A>)->u16{let a:&dyn A=&x.1;a.a()}fn main(){let w=W(2,1000u64);check(read(&w)==1000);}",
        false,
    ),
    (
        "tail-pattern",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<dyn A>)->u16{let W(tag,value)=x;value.a()+*tag as u16}fn main(){let w=W(2,1000u64);check(read(&w)==1002);}",
        false,
    ),
    (
        "tail-nested-pattern",
        "trait A{fn a(&self)->u16;}impl A for u8{fn a(&self)->u16{*self as u16}}#[repr(C)]struct I<T:?Sized>(u64,T);#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<I<dyn A>>)->u16{let W(tag,I(prefix,value))=x;value.a()+*tag as u16+*prefix as u16}fn main(){let w=W(2,I(1000,7u8));check(read(&w)==1009);}",
        false,
    ),
    (
        "tail-transparent",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(transparent)]struct W<T:?Sized>((),T);fn read(x:&W<dyn A>)->u16{x.1.a()}fn main(){let w=W((),1000u64);check(read(&w)==1000);}",
        false,
    ),
    (
        "tail-explicit-inner-alignment",
        "trait A{fn a(&self)->u16;}impl A for u8{fn a(&self)->u16{*self as u16}}#[repr(C,align(32))]struct I<T:?Sized>(u8,T);#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<I<dyn A>>)->u16{x.1.1.a()+x.0 as u16+x.1.0 as u16}fn main(){let w=W(2,I(3,7u8));check(read(&w)==12);}",
        false,
    ),
    (
        "tail-constant",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);const P:&W<dyn A>=&W(2,1000u64);fn main(){check(P.1.a()==1000&&P.0==2);}",
        false,
    ),
    (
        "tail-promoted-constructor",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>{tag:u8,value:T}fn make()->&'static W<dyn A>{&W{tag:2,value:1000u64}}fn main(){let p=make();check(p.value.a()==1000&&p.tag==2);}",
        false,
    ),
    (
        "tail-nested-constant-constructor",
        "trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);const P:&W<W<dyn A>>=&W(2,W(3,1000u64));fn main(){check(P.1.1.a()==1000&&P.0==2&&P.1.0==3);}",
        false,
    ),
    (
        "tail-upcast",
        "trait A{fn a(&self)->u16;}trait B:A{}impl A for u64{fn a(&self)->u16{*self as u16}}impl B for u64{}#[repr(C)]struct W<T:?Sized>(u8,T);fn base(x:&W<dyn B>)->&W<dyn A>{x}fn main(){let w=W(2,1000u64);let p=base(&w);check(p.1.a()==1000&&p.0==2);}",
        false,
    ),
];

#[test]
fn dynamic_tails_lower() -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    for &(name, source, _) in CASES {
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
fn invalid_object_tails_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "tail-packed-object",
            "#[repr(packed)]struct W<T:?Sized>(u8,T);trait A{}impl A for u8{}fn main(){let w=W(2,7u8);let p:&W<dyn A>=&w;}",
        ),
        (
            "tail-constant-missing-implementation",
            "trait A{}impl A for u16{}struct W<T:?Sized>(u8,T);const P:&W<dyn A>=&W(2,7u8);fn main(){}",
        ),
        (
            "tail-explicit-constructor-type",
            "trait A{}impl A for u16{}struct W<T:?Sized>(u8,T);const P:&W<dyn A>=&W::<u16>(2,7u8);fn main(){}",
        ),
        (
            "tail-borrowed-object-static",
            "trait A{}impl A for &u8{}struct W<T:?Sized>(u8,T);fn bad<'a>(x:&'a u8){let p:&W<dyn A+'static>=&W(2,x);}fn main(){}",
        ),
    ] {
        super::rustc::qualify(name, source, false)?;
        assert!(super::build(source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn dynamic_tails_execute() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("pa24-pa25-work/native-object-tails", CASES)
}
