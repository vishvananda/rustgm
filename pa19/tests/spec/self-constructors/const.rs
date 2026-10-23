fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<const N:usize>([u8;N]);impl<const N:usize>S<N>{fn make(x:[u8;N])->Self{Self(x)}}fn main(){check(S::make([3,7]).0[1]==7);}
