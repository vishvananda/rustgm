trait A<'a>{type Out;}fn f<T>()where for<'a>T:A<'a>{let _:T::Out;}fn main(){}
