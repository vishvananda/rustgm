fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum Opt<T>{Some(T),None}
impl<T> Opt<T>{fn map_or<U>(self,default:U,f:fn(T)->U)->U{match self{Opt::Some(t)=>f(t),Opt::None=>default}}}
fn whole(a:&[u8;2])->&[u8]{a}
fn get(o:Opt<&[u8;2]>)->&[u8]{o.map_or(&[],whole)}
fn main(){let a=[1u8,2];check(get(Opt::Some(&a)).len()==2);check(get(Opt::None).len()==0);}
