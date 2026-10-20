trait A<T>{}trait B:A<u8>{}fn bad(x:&dyn B)->&dyn A<u16>{x}fn main(){}
