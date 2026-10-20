fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn a(&self)->u8;}impl A for &u8{fn a(&self)->u8{**self}}struct H<'a,T:?Sized+'a>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){let v=7u8;let r=&v;let h=f(&r);check(h.r.a()==7);}
