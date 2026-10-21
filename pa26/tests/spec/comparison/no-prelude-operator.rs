#![no_implicit_prelude]
extern crate core;struct S;impl core::cmp::PartialEq for S{fn eq(&self,_:&Self)->bool{true}}fn main(){let _=S==S;}