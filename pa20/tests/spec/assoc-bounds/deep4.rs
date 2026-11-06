#![recursion_limit="512"]
trait A{type Out;}trait End{}trait T0{type Out;}trait T1{type Out;}trait T2{type Out;}trait T3{type Out;}fn need<T:End>(){}fn f<T:A<Out:T0<Out:T1<Out:T2<Out:T3<Out:End>>>>>>(){need::<<<<<<T as A>::Out as T0>::Out as T1>::Out as T2>::Out as T3>::Out>();}fn main(){}
