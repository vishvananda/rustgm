fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn need<T:Eq+?Sized>(_:&T){}fn main(){let a:[f32;0]=[];need(&a);}
