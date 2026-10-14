fn f<'a,T:'a>(x:&'a mut T)->(&'a mut T,&'a mut T)where &'a mut T:Copy{(x,x)}
fn main(){}
