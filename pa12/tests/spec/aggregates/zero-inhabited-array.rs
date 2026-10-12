fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
enum Void{}struct Z([Void;0]);fn f(z:Z)->Z{z}fn main(){let z=f(Z([]));let _=z.0;check(true);}
