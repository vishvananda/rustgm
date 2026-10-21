const X:u32={let x=7u32;let y=37u32;let mut r=&x;let p=&raw mut r;unsafe{*p=&y;}*r};fn main(){let _=X;}
