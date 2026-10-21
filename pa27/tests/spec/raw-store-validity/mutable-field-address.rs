struct S{x:u32}const X:u32={let mut s=S{x:7};let r=&mut s;let p=&raw mut r.x;unsafe{*p=9;}s.x};fn main(){let _=X;}
