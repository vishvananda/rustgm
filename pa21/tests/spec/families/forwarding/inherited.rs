trait Lending{type Ref<'a> where Self:'a;}trait Shared:Lending{}impl<F:Shared+?Sized>Lending for &F{type Ref<'a>=F::Ref<'a> where Self:'a;}fn main(){}
