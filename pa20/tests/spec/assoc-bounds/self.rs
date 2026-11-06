trait A{type Out;}trait B:A{}trait F{type Out;fn get()->Self::Out;}struct W<P>(P);impl<P:B<Out:F>>F for W<P>{type Out=<<P as A>::Out as F>::Out;fn get()->Self::Out{loop{}}}fn main(){}
