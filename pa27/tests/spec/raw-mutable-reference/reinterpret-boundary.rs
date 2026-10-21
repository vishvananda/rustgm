const X:u32={let mut x=7u32;let p=(&raw mut x)as *mut i32;unsafe{let _r=&mut *p;}37};fn main(){let _=X;}
