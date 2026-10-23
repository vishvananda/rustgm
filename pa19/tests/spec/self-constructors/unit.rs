fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn make()->Self{Self}}fn main(){let _=S::make();}
