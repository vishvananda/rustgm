trait A{}struct H<'a,'b,T:?Sized+'a+'b>{a:&'a T,b:&'b T}fn f<'a>(x:&'a H<'a,'a,dyn A+'a>)->&'a H<'a,'a,dyn A+'_>{x}fn main(){}
