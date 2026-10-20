trait A<'a>{}impl<'a>A<'a>for u8{}struct S<T>where for<'a>T:A<'a>{x:T}fn main(){let s=S{x:7u8};if s.x!=7{loop{}}}
