#![feature(const_trait_impl)]
pub const trait Access{type Item;fn access(&self)->&Self::Item;}
pub struct Smart<T>(T);
const impl<T> Access for Smart<T>{type Item=T;fn access(&self)->&T{&self.0}}
pub const fn field<T:Access>(x:&T)->&T::Item{Access::access(x)}
