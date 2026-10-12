fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
struct S(u8);macro_rules! make{($e:expr)=>{S($e)}}fn main(){let x=make!(4+5);check(x.0==9);}
