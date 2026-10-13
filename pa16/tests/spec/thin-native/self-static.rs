fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct N{v:u64,next:&'static N}static A:N=N{v:7,next:&A};fn main(){check(A.next.next.v==7);}
