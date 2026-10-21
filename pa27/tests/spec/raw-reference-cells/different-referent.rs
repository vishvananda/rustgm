const X:i32={let x=7u32;let r=&x;let p=(&raw const r)as *const &i32;unsafe{**p}};fn main(){let _=X;}
