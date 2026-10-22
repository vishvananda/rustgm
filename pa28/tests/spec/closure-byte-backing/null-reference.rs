static VALUE:u32=37;fn main(){let f=const{let r=&VALUE;let mut f=move||*r;unsafe{core::ptr::write_bytes(&raw mut f,0,1)};f};let _=f;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
