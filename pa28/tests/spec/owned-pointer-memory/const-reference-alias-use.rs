struct P<'a>{r:&'a mut u32}const X:u32={let mut n=7u32;let p=P{r:&mut n};let raw=&raw const p;let old=p;let q=unsafe{core::ptr::read(raw)};*old.r=11;*q.r};fn main(){let _=X;}
