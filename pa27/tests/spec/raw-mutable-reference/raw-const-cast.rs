const X:u32={let mut x=7u32;let p=(&raw const x)as *mut u32;unsafe{let r=&mut *p;*r=37;}x};fn main(){let _=X;}
