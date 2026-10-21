static mut S:u32=7;const X:u32={let p=&raw mut S;unsafe{*p=9;}0};fn main(){let _=X;}
