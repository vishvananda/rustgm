type A<T>=for<'a> fn(&'a u8,T)->T;fn f<'b>(x:A<&'b u8>)->for<'a> fn(&'a u8,&'b u8)->&'b u8{x}
fn main(){}
