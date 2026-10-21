union U{a:u8,b:bool}const X:bool={let mut u=U{a:2};unsafe{core::ptr::write(&raw mut u.b,true);core::ptr::read(&raw const u.b)}};fn main(){let _=X;}
