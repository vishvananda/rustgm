trait A{}impl<'a>A for &'a u8{}type H<'a,T:'a> = &'a T;fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{x}fn main(){}
