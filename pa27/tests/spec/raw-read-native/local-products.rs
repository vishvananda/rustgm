mod case0{const X:u32={let x=37;let p=&raw const x;unsafe{*p}};
pub fn value()->u32{X}}mod case1{const X:u32={let mut x=37;let p=&raw mut x;unsafe{*p}};
pub fn value()->u32{X}}mod case2{const X:u32={let x=(7u8,37u32);let p=&raw const x.1;unsafe{*p}};
pub fn value()->u32{X}}mod case3{const X:u32={let x=[7u32,37];let p=&raw const x[1];unsafe{*p}};
pub fn value()->u32{X}}mod case4{const X:u32={let x=(7u8,37u32);let p=&raw const x;unsafe{(*p).1}};
pub fn value()->u32{X}}mod case5{const X:u32={let x=[7u32,37];let p=&raw const x;unsafe{(*p)[1]}};
pub fn value()->u32{X}}fn main(){check(case0::value()==37);check(case1::value()==37);check(case2::value()==37);check(case3::value()==37);check(case4::value()==37);check(case5::value()==37);}
