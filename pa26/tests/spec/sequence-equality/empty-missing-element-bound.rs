fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct X;fn main(){let a:[X;0]=[];let b:[X;0]=[];let _=a==b;}
