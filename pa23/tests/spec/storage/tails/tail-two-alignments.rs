fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}impl A for u8{fn a(&self)->u16{*self as u16}}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<dyn A>)->u16{x.1.a()+x.0 as u16}fn main(){let a=W(2,7u8);let b=W(3,1000u64);check(read(&a)==9&&read(&b)==1003);}
