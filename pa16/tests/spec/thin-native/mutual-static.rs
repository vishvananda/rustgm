fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct N{v:u64,next:&'static N}static A:N=N{v:7,next:&B};static B:N=N{v:9,next:&A};fn main(){check(A.next.v==9 && B.next.v==7 && A.next.next.v==7);}
