use core::option::Option as Maybe;fn get(x:Maybe<u8>)->u8{match x{Maybe::Some(x)=>x,Maybe::None=>0}}fn main(){let _=get(Maybe::Some(7));}
