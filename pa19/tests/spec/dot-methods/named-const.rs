fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const N:usize=2;struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn main(){check(S.f::<N>()[1]==7);}