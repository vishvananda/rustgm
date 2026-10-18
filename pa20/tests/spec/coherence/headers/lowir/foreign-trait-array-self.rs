fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl Clone for [S;1]{fn clone(&self)->Self{[S]}}trait A{type Out;}impl A for u8{type Out=[S;1];}fn main(){}
