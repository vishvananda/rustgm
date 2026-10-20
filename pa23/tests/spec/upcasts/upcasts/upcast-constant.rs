fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}trait B:A{}impl A for u8{fn a(&self)->u8{*self}}impl B for u8{}const P:&dyn B=&7u8;const Q:&dyn A=P;fn main(){check(Q.a()==7);}
