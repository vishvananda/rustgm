fn take<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{}fn f<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{take::<'a,T>(x);take::<'a,T>(x);}
fn main(){}
