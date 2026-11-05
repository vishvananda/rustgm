trait Lending{type Ref<'a>where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}struct X;impl Lending for X{type Ref<'a>=u32 where Self:'a;fn get<'a>(&'a self)->Self::Ref<'static>{23}}fn main(){}
