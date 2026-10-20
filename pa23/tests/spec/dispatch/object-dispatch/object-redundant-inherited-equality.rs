trait A{type V;}trait B:A<V=u8>{}fn f(x:&dyn B)->&dyn B<V=u8>{x}fn main(){}
