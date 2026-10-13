fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let xs:[N;0]=[];let[a@..]=xs;let[b@..]=xs;let _=(a,b);let ys=xs;let _=ys;check(true);}