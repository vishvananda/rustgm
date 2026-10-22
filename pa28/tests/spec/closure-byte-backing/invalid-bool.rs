fn main(){let f=const{let b=true;let mut f=move||b;unsafe{core::ptr::write_bytes(&raw mut f,2,1)};f};let _=f;}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
