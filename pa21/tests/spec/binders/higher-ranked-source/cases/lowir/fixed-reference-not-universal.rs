trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn f<'x,T>()where &'x T:for<'a>A<'a>{}fn run<'x>(x:&'x u8){f::<'x,u8>();}fn main(){let n=1;run(&n);}
