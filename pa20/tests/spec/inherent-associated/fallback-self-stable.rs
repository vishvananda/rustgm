trait Get{type Type;fn get(value:u32)->u32;}struct S;impl Get for S{type Type=u32;fn get(value:u32)->u32{let actual:Self::Type=value;actual}}
