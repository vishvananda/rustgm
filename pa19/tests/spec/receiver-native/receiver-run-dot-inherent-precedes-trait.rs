fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f(&self)->u8{1}}trait A{fn f(&self)->u8;}impl A for S{fn f(&self)->u8{2}}fn main(){let x=S.f();check(x==1);}