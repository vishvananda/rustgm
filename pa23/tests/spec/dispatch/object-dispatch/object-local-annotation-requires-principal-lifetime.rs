trait A:'static{}impl A for u8{}fn f<'a>(x:&'a u8){let p:&'a(dyn A+'a)=x;}fn main(){}
