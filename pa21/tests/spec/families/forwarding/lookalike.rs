trait Lending{type Ref<'a> where Self:'a;}trait Other{type Ref<'a> where Self:'a;}impl<F:Other+?Sized>Lending for &F{type Ref<'a>=<F as Lending>::Ref<'a> where Self:'a;}fn main(){}
