trait A<'a>{type Out;}fn f<T>(x:T::Out)->T::Out where for<'a>T:A<'a>{x}fn main(){}
