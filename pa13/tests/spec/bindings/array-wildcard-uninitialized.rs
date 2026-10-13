fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
struct N(u8);fn main(){let a:[N;2];let[_,_]=a;let b:[N;3];let[..]=b;let z:[N;0];let[]=z;check(true);}