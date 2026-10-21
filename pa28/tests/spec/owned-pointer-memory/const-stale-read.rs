struct P{a:u32}const X:u32={let raw={let p=P{a:7};&raw const p};unsafe{core::ptr::read(raw).a}};fn main(){let _=X;}
