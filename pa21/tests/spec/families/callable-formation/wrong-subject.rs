trait Lending{type Ref<'a>where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}fn bad<'a,F:Lending,G:Lending>(x:&'a F)->G::Ref<'a>{F::get(x)}fn main(){}
