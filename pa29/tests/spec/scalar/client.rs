fn check(ok:bool){if !ok{let _=fail(0);}}
fn fail(n:u8)->u8{1/n}
fn main(){check(one::value(10)==14);check(two::value(10)==22);check(one::nested::twice(4)==10);check(one::small(7)==7);check(two::small(9)==9);check(one::signed(-12345678901234567890i128)==-12345678901234567890i128);check(one::truth(true));check(one::character('é')=='é');check(one::real(3.5)==3.5);check(two::real(1.25)==1.25);let f:fn(u32)->u32=one::value;check(f(3)==7);}
