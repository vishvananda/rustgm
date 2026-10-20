fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=make();need(&x);check(x.f()==7);}fn need<T:Send>(_:&T){}trait A{fn f(&self)->u8;}impl A for u8{fn f(&self)->u8{*self}}const C:(u8,u8)=(1,2);fn make()->impl A{match (1u8,2u8){C=>7u8,_=>9}}
