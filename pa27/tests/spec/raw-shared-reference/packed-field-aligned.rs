#[repr(C,packed)]struct P{x:u32}#[repr(C)]struct A{prefix:u64,p:P}const X:u32={let x=A{prefix:0,p:P{x:37}};let p=&raw const x.p.x;unsafe{let r=&*p;*r}};fn main(){let _=X;}
