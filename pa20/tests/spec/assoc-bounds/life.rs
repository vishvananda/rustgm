trait A{type Out;}fn want<'a,T:'a>(){}fn f<'a,T:A<Out:'a>>(){want::<'a,T::Out>();}fn main(){}
