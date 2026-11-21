fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{type Item;fn get(&self)->Self::Item;}struct Pair{v:[u8;2]}struct First<'a>(&'a [u8;2]);impl<'a>Get for First<'a>{type Item=&'a u8;fn get(&self)->&'a u8{&self.0[0]}}
fn first(v:&[u8;2])->impl Get<Item=&u8>{First(v)}fn main(){let p=Pair{v:[5,6]};let f=first(&p.v);check(*f.get()==5);}
