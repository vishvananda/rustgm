struct P{a:u32}const X:u32={let mut p=P{a:7};let raw=&raw mut p;let old=p;unsafe{core::ptr::write(raw,P{a:37});let q=core::ptr::read(raw);q.a+old.a}};fn main(){let _=X;}
