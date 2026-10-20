trait A{}trait B:A{}fn bad<'a>(x:&'a(dyn B+'a))->&'a(dyn A+'static){x}fn main(){}
