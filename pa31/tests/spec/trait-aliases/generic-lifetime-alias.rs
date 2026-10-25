#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait Read<V>{fn read(&self,value:V)->V;}
impl Read<u8> for u8{fn read(&self,value:u8)->u8{value}}
trait Borrow<'a,V> = Read<V> + 'a;
fn read<'a,T:Borrow<'a,u8>>(value:&'a T)->u8{value.read(7)}
fn main(){println!("{}",read(&0_u8));}
