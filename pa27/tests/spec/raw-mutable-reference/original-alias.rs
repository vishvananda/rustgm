const X:u32={let mut x=7u32;let p=&raw mut x;unsafe{let s=&*p;let _y=*s;let r=&mut *p;*r=37;}x};fn main(){let _=X;}
