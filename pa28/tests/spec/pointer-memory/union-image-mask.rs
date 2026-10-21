#[derive(Copy,Clone)]union U{a:u8,b:u32}const X:u32={let mut u=U{a:7};unsafe{let v=core::ptr::read(&raw const u);core::ptr::write(&raw mut u,v);core::ptr::read(&raw const u.b)}};fn main(){let _=X;}
