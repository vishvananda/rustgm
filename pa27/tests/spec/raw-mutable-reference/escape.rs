const X:&mut u32={let mut x=7u32;let p=&raw mut x;unsafe{&mut *p}};fn main(){let _=X;}
