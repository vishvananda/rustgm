trait A<'a>{}impl A<'static>for u8{}struct S<T>where for<'a>T:A<'a>{x:T}fn main(){let s=S{x:7u8};}
