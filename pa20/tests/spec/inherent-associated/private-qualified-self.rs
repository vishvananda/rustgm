#![feature(inherent_associated_types)]
mod hidden{pub struct S;impl S{type Type=u32;}}trait Get{type Type;fn get(value:u32)->u32;}impl Get for hidden::S{type Type=u32;fn get(value:u32)->u32{let actual:<Self>::Type=value;actual}}
