const X:u32={let x=7u32;let r=&x;let p=(&raw const r)as *const &mut u32;unsafe{let _outer=&*p;}37};fn main(){let _=X;}
