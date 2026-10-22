unsafe extern "C"{fn promoted(tag:u32,...)->u32;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=7u8;let b=-3i16;let c=1.5f32;check(unsafe{promoted(0,a as i32,b as i32,true as i32,c as f64)}==1);}
