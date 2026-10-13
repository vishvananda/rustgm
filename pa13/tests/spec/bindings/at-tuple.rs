fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn main(){let whole@(x,(y,z))=(1u8,(2u16,3u32));check(whole.0==x&&whole.1.0==y&&z==3);}