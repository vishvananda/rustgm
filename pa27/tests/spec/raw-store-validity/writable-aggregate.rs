const X:u32={let mut x=7u32;let t=(&raw mut x,);unsafe{*t.0=9;}x};fn main(){let _=X;}
