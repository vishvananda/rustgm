trait Lending{type Ref<'a> where Self:'a;}impl<F:Lending+?Sized>Lending for &F{type Ref<'a>=F::Ref<'a> where Self:'a;}fn main(){}
