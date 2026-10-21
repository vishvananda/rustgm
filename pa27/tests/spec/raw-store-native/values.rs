const A:u32={let mut x=7u32;let p=&raw mut x;let r=&x;unsafe{*p=19;}let v=*r;v+x};
const B:u32={let x=7u32;let p=(&raw const x) as *mut u32;unsafe{*p=29;}x};
const C:(bool,char)={let mut b=false;let mut c='a';let p=&raw mut b;let q=&raw mut c;unsafe{*p=true;*q='λ';}(b,c)};
fn main(){check(A==38);check(B==29);check(C.0);check(C.1=='λ');}
