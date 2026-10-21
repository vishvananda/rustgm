const X:u32={let mut x=7u32;let p=&raw mut x;unsafe{*p=37;}x};fn main(){let _=X;}
