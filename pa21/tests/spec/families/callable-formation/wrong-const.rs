trait L{type Ref<'a,const N:usize>where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a,3>;}fn bad<'a,F:L>(x:&'a F)->F::Ref<'a,4>{F::get(x)}fn main(){}
