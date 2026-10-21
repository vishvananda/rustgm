#[derive(Copy,Clone)]union U{a:u8,b:bool}const X:u8={let mut u=U{a:2};unsafe{let v=core::ptr::read(&raw const u);core::ptr::write(&raw mut u,v);u.a}};fn main(){let _=X;}
