#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait Any=;fn any<T:Any>(){}fn main(){any::<u8>();}
