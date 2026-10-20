fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(&self)->u8;}struct W<'a>(&'a mut u8);impl<'a>A for W<'a>{fn f(&self)->u8{*self.0}}impl<'a>Drop for W<'a>{fn drop(&mut self){*self.0+=2;}}fn read(x:&dyn A)->u8{x.f()}fn main(){let mut v=7u8;{let w=W(&mut v);check(read(&w)==7);}check(v==9);}
