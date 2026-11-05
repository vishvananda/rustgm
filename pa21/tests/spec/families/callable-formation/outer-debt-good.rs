struct Need<'b,T:'b>(&'b T);trait A{type G<'a,'b>where Self:'b;}fn item<'b,T:A+'b>(){let _:for<'a>fn(&'a T,T::G<'a,'b>);}fn main(){}
