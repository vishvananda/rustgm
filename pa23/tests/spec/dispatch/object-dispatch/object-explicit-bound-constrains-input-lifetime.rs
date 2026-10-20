trait A:'static{}fn f<'a>(x:&'a(dyn A+'a))->&'static dyn A{x}fn main(){}
