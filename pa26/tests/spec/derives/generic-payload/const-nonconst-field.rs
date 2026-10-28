#![feature(derive_const,const_trait_impl,const_clone)]
struct Leaf;impl Clone for Leaf{fn clone(&self)->Self{Leaf}}#[derive_const(Clone)]struct Wrap<T>(T);const VALUE:Wrap<Leaf>=Wrap(Leaf).clone();fn main(){}
