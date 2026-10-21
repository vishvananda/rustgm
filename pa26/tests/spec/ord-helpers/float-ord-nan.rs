fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=0.0f32/0.0;let _=Ord::clamp(n,0.0,1.0);}