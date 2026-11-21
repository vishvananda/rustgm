trait Get{type Item;fn get(&self)->Self::Item;}struct Fixed;impl Get for Fixed{type Item=&'static u8;fn get(&self)->&'static u8{&7}}
fn fixed()->impl Get<Item=&u8>{Fixed}fn main(){}
