trait A{}struct P(*const u8);impl A for P{}trait B:Send{}impl<T:Send>B for T{}fn outer(x:*const u8)->impl B{inner(x)}fn inner(x:*const u8)->impl A{P(x)}
fn main(){}
