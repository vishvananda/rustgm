fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(&'static u8);fn main(){let a:[S;0]=[];let _=a;check(true);}
