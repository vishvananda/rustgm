trait Lending{type Ref<'a> where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}fn bad<'a,F:Lending+?Sized+'static>(x:&'a F)->F::Ref<'static>{F::get(x)}fn main(){}
