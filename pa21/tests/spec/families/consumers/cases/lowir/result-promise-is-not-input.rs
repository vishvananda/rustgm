fn needs<T:'static>(){}trait Q{type Out<T>:'static;fn f<T>(x:Self::Out<T>){needs::<T>();}}
fn main(){}
