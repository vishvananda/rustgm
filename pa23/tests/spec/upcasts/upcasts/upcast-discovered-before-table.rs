fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}fn base(x:&dyn B)->&dyn A{x}fn make(x:&u8)->&dyn B{x}fn main(){let f=base as for<'a>fn(&'a(dyn B+'a))->&'a(dyn A+'a);let v=7u8;check(f(make(&v)).a()==7);}
