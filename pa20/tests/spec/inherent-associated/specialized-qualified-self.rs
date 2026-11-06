#![feature(inherent_associated_types)]
struct S<T>(T);impl S<u8>{type Type=u8;}impl S<u16>{type Type=u16;}trait Get{type Type;fn get(value:u32)->u32;}impl Get for S<u32>{type Type=u32;fn get(value:u32)->u32{let actual:<Self>::Type=value;actual}}
