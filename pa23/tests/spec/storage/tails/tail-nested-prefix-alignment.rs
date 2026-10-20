fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}impl A for u8{fn a(&self)->u16{*self as u16}}#[repr(C)]struct I<T:?Sized>(u64,T);#[repr(C)]struct W<T:?Sized>(u8,T);fn read(x:&W<I<dyn A>>)->u16{x.1.1.a()+x.0 as u16+x.1.0 as u16}fn main(){let w=W(2,I(1000,7u8));check(read(&w)==1009);}
