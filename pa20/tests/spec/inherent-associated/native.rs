#![feature(inherent_associated_types)]
struct S;impl S{type A=u8;fn identity(value:Self::A)->Self::A{value}}#[unsafe(no_mangle)]pub extern "C" fn alias_identity(value:S::A)->u8{S::identity(value)}
