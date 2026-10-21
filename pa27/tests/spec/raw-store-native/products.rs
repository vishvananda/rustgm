struct S{a:u32,b:u32}const fn write(p:*mut u32,v:u32){unsafe{*p=v;}}
const A:S={let mut s=S{a:7,b:8};let t=(&raw mut s,);unsafe{write(&raw mut (*t.0).b,43);}s};
const B:[u32;2]={let mut a=[1,2];let p=&raw mut a;unsafe{(*p)[1]=47;}a};
const C:(u32,u32)={let mut x=(1,2);let p=&raw mut x;unsafe{*p=(53,59);}x};
fn main(){check(A.a==7);check(A.b==43);check(B[0]==1);check(B[1]==47);check(C.0==53);check(C.1==59);}
