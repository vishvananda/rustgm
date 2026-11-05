trait L{type Ref<'a,T>where Self:'a,T:'a;fn get<'a>(&'a self)->Self::Ref<'a,fn(&'a u8)>;}fn bad<'a,F:L>(x:&'a F)->F::Ref<'a,for<'b>fn(&'b u8)>{F::get(x)}fn main(){}
