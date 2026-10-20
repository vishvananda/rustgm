fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let w=W(make());let p:&(dyn A+Send)=&w.0;check(p.f()==9);}struct W<T:Send>(T);trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}fn make()->impl A{9u8}
