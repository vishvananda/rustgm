#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A{}trait B{}trait Both=A+B;fn use_dyn(_: &dyn Both){}fn main(){}
