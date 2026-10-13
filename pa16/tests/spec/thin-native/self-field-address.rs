fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{v:u64,r:&'static u64}static A:S=S{v:42,r:&A.v};fn main(){check(*A.r==42);}
