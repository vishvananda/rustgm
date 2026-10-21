struct P{a:u32}const X:u32={let mut p=P{a:7};unsafe{let q=core::ptr::read(&raw const p);core::ptr::write(&raw mut p,P{a:37});q.a+p.a}};fn main(){let _=X;}
