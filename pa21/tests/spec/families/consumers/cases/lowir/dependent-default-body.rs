fn needs<T:'static>(){}trait Q{type Out<T> where for<'a>T:'a;fn f<T>(x:Self::Out<T>){needs::<T>();}}
fn main(){}
