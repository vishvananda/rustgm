fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<const N:usize>(&self)->[u8;N]{[7;N]}}fn g<const N:usize>()->[u8;N]{S.f::<N>()}fn main(){check(g::<2>()[1]==7);}