mod case0{#[repr(C,packed)]struct P{prefix:u8,x:u32}const X:u32={let x=P{prefix:0,x:37};let p=&raw const x;unsafe{let r=&*p;r.x}};pub fn value()->u32{X}
}mod case1{#[repr(C,packed)]struct P{x:u32}#[repr(C)]struct A{prefix:u64,p:P}const X:u32={let x=A{prefix:0,p:P{x:37}};let p=&raw const x.p.x;unsafe{let r=&*p;*r}};pub fn value()->u32{X}
}mod case2{const X:u32={let x=();let p=&raw const x;unsafe{let _r=&*p;}37};pub fn value()->u32{X}
}fn main(){check(case0::value()==37);check(case1::value()==37);check(case2::value()==37);}
