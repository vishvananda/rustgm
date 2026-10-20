fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}impl A for u64{fn a(&self)->u16{*self as u16}}#[repr(transparent)]struct W<T:?Sized>((),T);fn read(x:&W<dyn A>)->u16{x.1.a()}fn main(){let w=W((),1000u64);check(read(&w)==1000);}
