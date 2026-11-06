trait A{type Out;}fn want<'a,T:'a>(){}fn f<'a,'b,T:A<Out:'a>>(){want::<'b,T::Out>();}fn main(){}
