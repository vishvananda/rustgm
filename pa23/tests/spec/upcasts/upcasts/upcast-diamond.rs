fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}trait B:A{fn b(&self)->u8{2}}trait C:A{fn c(&self)->u8{3}}trait D:B+C{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}impl C for u8{}impl D for u8{}fn main(){let v=7u8;let d:&dyn D=&v;let b:&dyn B=d;let c:&dyn C=d;let a:&dyn A=c;check(b.b()==2&&c.c()==3&&a.a()==7);}
