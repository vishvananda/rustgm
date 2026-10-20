trait A<'a>{}fn bad<'s>(x:&'s dyn A<'static>)->&'s dyn for<'a>A<'a>{x}fn main(){}
