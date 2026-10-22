fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct N;fn main(){let a:[N;0]=[];let b:[N;0]=[];let _=a<b;}