fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Lending{type Ref<'a>where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>;}impl<F:Lending+?Sized>Lending for &F{type Ref<'a>=F::Ref<'a> where Self:'a;fn get<'a>(&'a self)->Self::Ref<'a>{F::get(*self)}}
struct S{value:u32}impl Lending for S{type Ref<'a>=&'a u32;fn get<'a>(&'a self)->Self::Ref<'a>{&self.value}}fn main(){let s=S{value:17};let view=&s;let p=<&S as Lending>::get(&view);check(*p==17);}
