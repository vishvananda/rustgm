#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl !A for u32{}#[rustc_strict_coherence]struct S<T>(T);impl<T:A>S<T>{fn value(&self){}}impl S<u32>{fn value(&self){}}fn main(){}
