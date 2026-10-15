fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<const N:usize>;impl S<1>{fn f(&self)->u8{7}}impl S<2>{fn f(&self)->u8{9}}fn main(){check(S::<1>.f()==7);check(S::<2>.f()==9);}