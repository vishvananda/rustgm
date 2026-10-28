#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]#[repr(i16)]enum Value<T>{Low(T)=-7,Unit=27,Record{field:T}=300}
fn main(){let a=Value::Low(9u8);let b=Value::Low(10u8);let c=Value::Record{field:9u8};println!("{} {} {} {} {:?} {:?}",a==a,a==c,a<b,a<Value::Unit,c,a);}
