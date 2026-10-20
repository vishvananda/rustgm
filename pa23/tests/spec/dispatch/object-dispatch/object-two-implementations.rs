fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}impl A for u16{fn f(&self)->u8{(*self+2)as u8}}fn read(x:&dyn A)->u8{x.f()}fn main(){let a=7u8;let b=8u16;check(read(&a)==7&&read(&b)==10);}
