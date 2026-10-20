trait A{}impl A for &u8{}fn f<'a>(x:&'a &'a u8){let p:&dyn A=x;let q:&'a(dyn A+'a)=p;}fn main(){}
