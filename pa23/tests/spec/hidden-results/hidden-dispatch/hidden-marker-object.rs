fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=make();let p:&(dyn A+Send)=&x;check(p.f()==7);}trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn make()->impl A{7u8}
