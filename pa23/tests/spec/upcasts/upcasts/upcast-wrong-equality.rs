trait A{type V;}trait B:A<V=u8>{}fn bad(x:&dyn B)->&dyn A<V=u16>{x}fn main(){}
