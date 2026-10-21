const X:u32={let mut x=7u32;let p=&raw mut x;let r=&x;unsafe{*p=9;}let _=*r;x};fn main(){let _=X;}
