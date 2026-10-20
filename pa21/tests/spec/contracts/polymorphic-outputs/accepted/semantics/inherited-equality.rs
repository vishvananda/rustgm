fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get{type Item;fn get(&self)->Self::Item;}trait Derived:Get<Item=u8>{}impl Get for u8{type Item=u8;fn get(&self)->u8{*self}}impl Derived for u8{}fn f()->impl Derived{29u8}fn main(){check(f().get()==29);}
