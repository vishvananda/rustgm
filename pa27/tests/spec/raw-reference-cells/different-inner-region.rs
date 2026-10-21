const X:u32={let x=7u32;let r=&x;let s=&r;let p=(&raw const s)as *const &'static &'static u32;unsafe{***p}};fn main(){let _=X;}
