fn pass<U>(f:fn(U)->U)->fn(U)->U{f}
fn take<'a>(x:(&'a u8,&'a u8))->(&'a u8,&'a u8){x}
fn bad<'a>(x:&'a u8)->&'static u8{let f=pass::<(_,_)>(take);f((x,x)).1}
fn main(){}

