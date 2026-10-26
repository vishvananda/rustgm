#![allow(dead_code,unused_imports,incomplete_features,internal_features,unused_macros)]
#![feature(specialization)]
trait Get{fn get(self)->u8;}impl Get for u8{fn get(self)->u8{self}}
trait V{fn f()->impl Get+Copy;}impl<T>V for T{default fn f()->u8{17u8}}impl V for u16{}fn main(){}
