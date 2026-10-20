fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);const P:&W<dyn A>=&W(2,1000u64);fn main(){check(P.1.a()==1000&&P.0==2);}
