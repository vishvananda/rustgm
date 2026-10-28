use std::hash::{Hash,Hasher};
struct Trace;impl Hasher for Trace{fn finish(&self)->u64{0}fn write(&mut self,bytes:&[u8]){for b in bytes{print!("{b:02x}");}print!(";");}}
#[derive(Hash)]#[repr(i16)]enum Value<T>{Low(T)=-7,Unit=27,Record{field:T}=300}
#[derive(Hash)]struct Collision<__H>{field:__H}
fn main(){Value::Low(5u8).hash(&mut Trace);println!();Value::<u8>::Unit.hash(&mut Trace);println!();Value::Record{field:7u8}.hash(&mut Trace);println!();Collision{field:9u8}.hash(&mut Trace);println!();}
