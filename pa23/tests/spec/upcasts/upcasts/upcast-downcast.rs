trait A{}trait B:A{}fn bad(x:&dyn A)->&dyn B{x}fn main(){}
