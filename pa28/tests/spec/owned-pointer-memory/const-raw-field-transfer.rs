struct P{r:*mut u32}const X:u32={let mut n=7u32;let p=P{r:&raw mut n};let raw=&raw const p;let _old=p;let q=unsafe{core::ptr::read(raw)};unsafe{*q.r}};fn main(){let _=X;}
