#[derive(PartialEq)]struct Fog;impl PartialOrd for Fog{fn partial_cmp(&self,_:&Self)->Option<std::cmp::Ordering>{print!("F;");None}}
#[derive(PartialEq,PartialOrd)]enum Value<T>{Low(T),Unit,High(T)}
fn main(){println!("{:?}",Value::Low(Fog).partial_cmp(&Value::Low(Fog)));println!("{:?}",Value::Low(Fog).partial_cmp(&Value::High(Fog)));}
