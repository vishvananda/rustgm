mod case0{static S:u32=37;const R:&u32={let p=&raw const S;unsafe{&*p}};const X:u32=*R;pub fn value()->u32{X}
}mod case1{static S:(u8,u32)=(7,37);const R:&u32={let p=&raw const S;unsafe{&(*p).1}};const X:u32=*R;pub fn value()->u32{X}
}fn main(){check(case0::value()==37);check(case1::value()==37);}
