fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T,const N:usize>([T;N]);fn main(){let a:S<u8,0>=S([]);let b=S([21u8]);check(b.0[0]==21);}
