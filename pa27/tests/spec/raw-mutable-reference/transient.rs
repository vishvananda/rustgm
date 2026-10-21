const fn f()->u32{let mut x=7u32;let p=&raw mut x;unsafe{let r=&mut *p;*r=37;}x}const X:u32=f();fn main(){let _=X;}
