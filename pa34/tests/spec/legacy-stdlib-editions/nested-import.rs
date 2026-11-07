#![allow(dead_code)]
mod nested { mod inner { pub struct Byte; } use inner::Byte; pub fn take(_: Byte) {} }
