#![feature(inherent_associated_types)]
trait Get{type Type;fn get(value:u8)->u8;}struct S;impl S{type Type=u8;}impl Get for S{type Type=u32;fn get(value:u8)->u8{let actual:<Self>::Type=value;actual}}
