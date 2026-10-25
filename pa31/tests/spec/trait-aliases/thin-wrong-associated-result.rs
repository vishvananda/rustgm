#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait Pointee {type Metadata;} trait PointeeSized {}
impl Pointee for u8 {type Metadata=u8;} impl PointeeSized for u8 {}
pub trait Thin = Pointee<Metadata=()> + PointeeSized;
fn thin<T:Thin>(){} fn main(){thin::<u8>();}
