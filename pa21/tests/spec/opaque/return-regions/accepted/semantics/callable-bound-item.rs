fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{type Item;fn get(&self)->Self::Item;}struct Pair{v:[u8;2]}struct First<'a>(&'a [u8;2]);impl<'a>Get for First<'a>{type Item=&'a u8;fn get(&self)->&'a u8{&self.0[0]}}
fn pick<F:Fn(&u8)->bool>(v:&[u8;2],f:F)->impl Get<Item=&u8>{let _=f(&v[0]);First(v)}fn yes(_:&u8)->bool{true}fn main(){let p=Pair{v:[5,6]};check(*pick(&p.v,yes).get()==5);}
