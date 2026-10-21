const X:u32={let x=37u32;let p=(&raw const x)as *const u8;unsafe{let r=&*p;*r as u32}};fn main(){let _=X;}
