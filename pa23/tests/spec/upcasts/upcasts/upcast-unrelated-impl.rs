trait A{}trait B{}impl B for dyn A+'_{}fn bad(x:&dyn A)->&dyn B{x}fn main(){}
