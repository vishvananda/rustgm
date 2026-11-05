trait Lending{type Ref<'a> where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}fn bad<F:Lending+'static>(x:F)->F::Ref<'static>{F::get(&x)}fn main(){}
