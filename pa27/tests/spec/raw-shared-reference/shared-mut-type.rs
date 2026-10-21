const X:u32={let x=37u32;let p=(&x as *const u32)as *mut u32;unsafe{let r=&*p;*r}};fn main(){let _=X;}
