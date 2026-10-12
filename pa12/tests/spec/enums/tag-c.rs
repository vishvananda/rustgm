fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(C)]enum E{A=-5,B=7} fn main(){check(E::A as i64 == -5 && E::B as u64==7);}
