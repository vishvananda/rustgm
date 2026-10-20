trait A{}impl<'a>A for &'a u8{}struct H<'a,T:?Sized+'a>{r:&'a T}fn f<'a>(x:&'a &'a u8)->H<'a,dyn A>{H{r:x}}fn main(){}
