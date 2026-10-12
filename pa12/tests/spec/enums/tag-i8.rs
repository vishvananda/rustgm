fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(i8)]enum E{A=-2,B=7,C} fn id(e:E)->E{e} fn main(){check(id(E::A) as i64 == -2 && E::B as u64==7 && E::C as u8==8);}
