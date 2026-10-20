fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>{tag:u8,value:T}fn make()->&'static W<dyn A>{&W{tag:2,value:1000u64}}fn main(){let p=make();check(p.value.a()==1000&&p.tag==2);}
