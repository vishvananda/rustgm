fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,X,T:?Sized+'a>{x:X,r:&'a T}fn main(){let v=7u8;let r=&v;let h:H<'_,_,dyn A>=H{x:2u8,r:&r};check(h.r.a()==7&&h.x==2);}
