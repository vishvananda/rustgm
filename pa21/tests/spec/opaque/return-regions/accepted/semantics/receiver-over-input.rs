fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{type Item;fn get(&self)->Self::Item;}struct Pair{v:[u8;2]}struct First<'a>(&'a [u8;2]);impl<'a>Get for First<'a>{type Item=&'a u8;fn get(&self)->&'a u8{&self.0[0]}}
impl Pair{fn first_after(&self,_k:&u8)->impl Get<Item=&u8>{First(&self.v)}}fn main(){let p=Pair{v:[5,6]};let k=1;let f=p.first_after(&k);check(*f.get()==5);}
