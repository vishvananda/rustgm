const fn write(p:*mut u32){unsafe{*p=9;}}const X:u32={let mut x=7u32;let p=&raw mut x;write(p);x};fn main(){let _=X;}
