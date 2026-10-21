mod case0{const unsafe fn read(p:*const u32)->u32{unsafe{*p}}const X:u32={let x=37;unsafe{read(&raw const x)}};
pub fn value()->u32{X}}mod case1{static S:u32=37;const X:u32={let p=&raw const S;unsafe{*p}};
pub fn value()->u32{X}}mod case2{static S:(u8,u32)=(7,37);const X:u32={let p=&raw const S.1;unsafe{*p}};
pub fn value()->u32{X}}mod case3{const fn pointer()->*const u32{&raw const S}const X:u32=unsafe{*pointer()};static S:u32=37;
pub fn value()->u32{X}}fn main(){check(case0::value()==37);check(case1::value()==37);check(case2::value()==37);check(case3::value()==37);}
