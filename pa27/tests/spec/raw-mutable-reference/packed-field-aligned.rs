#[repr(C,packed)]struct P{x:u32}#[repr(C)]struct A{prefix:u64,p:P}const X:u32={let mut x=A{prefix:0,p:P{x:7}};let p=&raw mut x.p.x;unsafe{let r=&mut *p;*r=37;}x.p.x};fn main(){let _=X;}
