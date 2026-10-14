fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T,const N:usize>([T;N]);fn main(){let a=S([3u16,5,9]);let b=S([4u8;5]);check(a.0[2]==9&&b.0[4]==4);}
