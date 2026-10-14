struct S<T,const N:usize>([T;N]);fn f<T:Copy>(x:T)->T{let s:S<T,_>=S([x;3]);s.0[2]}
fn main(){}
