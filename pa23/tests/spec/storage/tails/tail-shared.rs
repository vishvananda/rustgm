fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u16;}impl A for u16{fn a(&self)->u16{*self}}#[repr(C)]struct W<T:?Sized>{tag:u8,value:T}fn read(x:&W<dyn A>)->u16{x.value.a()+x.tag as u16}fn main(){let w=W{tag:2,value:1000u16};check(read(&w)==1002);}
