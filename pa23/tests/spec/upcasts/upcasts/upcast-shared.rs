fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}trait B:A{fn b(&self)->u8{9}}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}fn base(x:&dyn B)->&dyn A{x}fn main(){let v=7u8;let p:&dyn B=&v;check(base(p).a()==7);}
