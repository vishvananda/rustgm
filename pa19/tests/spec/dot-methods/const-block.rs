fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn main(){check(S.f::<{1usize+1}>()[1]==7);}