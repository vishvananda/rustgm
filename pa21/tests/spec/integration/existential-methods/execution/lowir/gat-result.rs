fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{type V<'a>where Self:'a;fn get<'a>(&'a self)->Self::V<'a>;}impl Get for u8{type V<'a>=&'a u8;fn get<'a>(&'a self)->&'a u8{self}}trait Make{fn make()->impl for<'a>Get<V<'a>=&'a u8>;}impl Make for u16{fn make()->impl for<'a>Get<V<'a>=&'a u8>{149u8}}fn main(){let x=<u16 as Make>::make();check(*x.get()==149);}
