struct D{a:u32}impl Drop for D{fn drop(&mut self){}}static S:D=D{a:37};const X:D=unsafe{core::ptr::read(&raw const S)};fn main(){let _=X;}
