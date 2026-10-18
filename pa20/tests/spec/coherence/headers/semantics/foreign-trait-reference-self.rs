fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl Clone for &S{fn clone(&self)->Self{*self}}trait A{type Out;}impl A for u8{type Out=S;}fn main(){}
