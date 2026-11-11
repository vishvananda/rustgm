#![feature(const_trait_impl)]
const trait Factory: Sized { fn make() -> Self; }
const fn create0<T: Factory>() -> fn() -> T { T::make }
const fn create1<T: Factory>() -> fn() -> T { T::make }
const fn create2<T: Factory>() -> fn() -> T { T::make }
const fn create3<T: Factory>() -> fn() -> T { T::make }
const fn create4<T: Factory>() -> fn() -> T { T::make }
const fn create5<T: Factory>() -> fn() -> T { T::make }
const fn create6<T: Factory>() -> fn() -> T { T::make }
const fn create7<T: Factory>() -> fn() -> T { T::make }
fn main(){}
