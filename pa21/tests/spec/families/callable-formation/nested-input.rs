trait Lending{type Ref<'a> where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}fn item<F:Lending+?Sized>(){let _:for<'a>fn(&'a F,for<'b>fn(&'b F,F::Ref<'a>));}fn main(){}
