trait L{type Ref<'a,'b>where Self:'a,Self:'b;fn get<'a>(&'a self)->Self::Ref<'a,'a>;}fn bad<'a,'b,F:L+'a+'b>(x:&'a F)->F::Ref<'a,'b>{F::get(x)}fn main(){}
