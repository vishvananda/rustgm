fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[derive(Clone,Copy)]#[repr(i16)]enum E{A=-7,B=99}fn make()->[E;4]{[E::A;4]}fn main(){let mut a=make();a[2]=E::B;check(a[0] as i64 == -7 && a[2] as u64==99 && a[3] as i16 == -7);}
