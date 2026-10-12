fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(i16)]enum E{A=if true{300.0f32 as i16}else{1/0},B=(0.0f64/0.0) as i16,C=(-1.0f32/0.0) as i16}fn main(){check(E::A as i16==300 && E::B as i16==0 && E::C as i16 == -32768);}
