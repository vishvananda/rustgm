#[repr(C,packed)]struct P{x:u32}#[repr(C)]struct A{a:u64,p:P}const X:u32={let x=A{a:0,p:P{x:37}};let p=&raw const x.p.x;unsafe{*p}};fn main(){let _=X;}
