trait A{}struct P(*const u8);impl A for P{}fn outer(x:*const u8)->impl A+Send{inner(x)}fn inner(x:*const u8)->impl A{P(x)}
fn main(){}
