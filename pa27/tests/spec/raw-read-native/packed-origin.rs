mod case0{#[repr(C,packed)]struct P{prefix:u8,x:u32}const X:u32={let x=P{prefix:0,x:37};let p=&raw const x;unsafe{(*p).x}};
pub fn value()->u32{X}}mod case1{#[repr(C,packed)]struct I{prefix:u8,x:u32}#[repr(C,packed)]struct P{prefix:u8,inner:I}const X:u32={let x=P{prefix:0,inner:I{prefix:0,x:37}};let p=&raw const x;unsafe{(*p).inner.x}};
pub fn value()->u32{X}}mod case2{#[repr(C,packed)]struct P{prefix:u8,p:*const u32}const X:u32={let x=37u32;let outer=P{prefix:0,p:&raw const x};let p=&raw const outer;unsafe{*((*p).p)}};
pub fn value()->u32{X}}fn main(){check(case0::value()==37);check(case1::value()==37);check(case2::value()==37);}
