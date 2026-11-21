trait Get{type Item;fn get(&self)->Self::Item;}struct Pair{v:[u8;2]}struct First<'a>(&'a [u8;2]);impl<'a>Get for First<'a>{type Item=&'a u8;fn get(&self)->&'a u8{&self.0[0]}}
fn first(v:&[u8;2],w:&[u8;2])->impl Get<Item=&u8>{let _=w;First(v)}fn main(){}
