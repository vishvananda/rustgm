fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let xs=[N(2)];let[_,a@..]=xs;let[_,b@..]=xs;let ys=xs;let _=(a,b);check(ys[0].0==2);}