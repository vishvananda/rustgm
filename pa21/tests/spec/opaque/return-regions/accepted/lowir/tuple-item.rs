fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{type Item;fn get(&self)->Self::Item;}struct Pair{v:[u8;2]}struct Both<'a>(&'a [u8;2]);impl<'a>Get for Both<'a>{type Item=(&'a u8,u8);fn get(&self)->(&'a u8,u8){(&self.0[0],self.0[1])}}
impl Pair{fn both(&self)->impl Get<Item=(&u8,u8)>{Both(&self.v)}}fn main(){let p=Pair{v:[5,6]};let (a,b)=p.both().get();check(*a==5);check(b==6);}
