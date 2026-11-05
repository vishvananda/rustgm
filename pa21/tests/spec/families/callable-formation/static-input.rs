trait Lending{type Ref<'a>where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}fn body<F:Lending+'static>(x:&'static F)->F::Ref<'static>{F::get(x)}fn main(){}
