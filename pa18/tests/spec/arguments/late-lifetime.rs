fn f<'a,T>(x:&'a T)->&'a T{x}fn main(){let _=f::<'static,u8>(&1);}
