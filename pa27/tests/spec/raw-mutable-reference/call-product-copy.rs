const fn pass(p:*mut u32)->*mut u32{p}const X:u32={let mut x=7u32;let p=&raw mut x;let pair=(p,0u32);let q=pass(pair.0);unsafe{let r=&mut *q;*r=37;}x};fn main(){let _=X;}
