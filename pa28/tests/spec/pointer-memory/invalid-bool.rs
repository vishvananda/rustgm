union U{a:u8,b:bool}const X:bool={let u=U{a:2};unsafe{core::ptr::read(&raw const u.b)}};fn main(){let _=X;}
