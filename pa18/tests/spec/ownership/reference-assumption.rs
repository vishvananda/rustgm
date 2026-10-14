fn f<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{let a=x;let b=x;}
fn main(){}
