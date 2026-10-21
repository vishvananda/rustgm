mod case0{const X:u32={let x=37u32;let p=(&x as *const u32)as *mut u32;unsafe{let r=&*p;*r}};pub fn value()->u32{X}
}mod case1{const X:u32={let mut x=7u32;let p=&raw mut x;unsafe{let r=&*p;let old=*r;*p=9;old+x}};pub fn value()->u32{X}
}fn main(){check(case0::value()==37);check(case1::value()==16);}
