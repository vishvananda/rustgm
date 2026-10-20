trait A{}trait B:A{}fn bad(x:&dyn B)->&mut dyn A{x}fn main(){}
