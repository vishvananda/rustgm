trait A{type Out;}fn want<'a,T:'a>(){}fn f<'a,T:A>(){want::<'a,T::Out>();}fn main(){}
