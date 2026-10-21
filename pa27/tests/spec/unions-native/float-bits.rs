fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]union U{x:u32,y:f32}fn main(){let u=U{x:1065353216};unsafe{check(u.y==1.0);}}