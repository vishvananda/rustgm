fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct N{v:u8,next:&'static N}static A:N=N{v:7,next:&B};static B:N=N{v:9,next:&A};const C:u8=A.next.next.v;const R:&u8=&A.next.v;fn main(){check(C==7 && *R==9);}
