mod case0{const X:u32={let x=37u32;let p=&raw const x;unsafe{let r=&*p;*r}};pub fn value()->u32{X}
}mod case1{const X:u32={let x=(7u8,37u32);let p=&raw const x;unsafe{let r=&*p;r.1}};pub fn value()->u32{X}
}mod case2{const X:u32={let x=[7u32,37];let p=&raw const x;unsafe{let r=&(*p)[1];*r}};pub fn value()->u32{X}
}mod case3{const fn borrow<'a>(r:&'a u32)->&'a u32{let p=r as *const u32;unsafe{&*p}}const X:u32={let x=37;*borrow(&x)};pub fn value()->u32{X}
}fn main(){check(case0::value()==37);check(case1::value()==37);check(case2::value()==37);check(case3::value()==37);}
