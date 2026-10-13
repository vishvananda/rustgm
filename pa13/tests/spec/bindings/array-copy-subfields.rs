fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let xs=[N(2),N(3)];let[N(a),_]=xs;let ys=xs;check(a==2&&ys[1].0==3);}