struct S{p:*const u32}const X:u32={let p={let x=7u32;&raw const x};let s=S{p};let p=&raw const s;unsafe{let outer=&*p;*outer.p}};fn main(){let _=X;}
