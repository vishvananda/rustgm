union U{a:u32,b:u32}const X:u32={let mut u=U{a:7};unsafe{let p=&raw const u.a;core::ptr::write(&raw mut u.b,37);core::ptr::read(p)}};fn main(){let _=X;}
