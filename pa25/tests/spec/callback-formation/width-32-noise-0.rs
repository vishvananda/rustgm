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
const fn create8<T: Factory>() -> fn() -> T { T::make }
const fn create9<T: Factory>() -> fn() -> T { T::make }
const fn create10<T: Factory>() -> fn() -> T { T::make }
const fn create11<T: Factory>() -> fn() -> T { T::make }
const fn create12<T: Factory>() -> fn() -> T { T::make }
const fn create13<T: Factory>() -> fn() -> T { T::make }
const fn create14<T: Factory>() -> fn() -> T { T::make }
const fn create15<T: Factory>() -> fn() -> T { T::make }
const fn create16<T: Factory>() -> fn() -> T { T::make }
const fn create17<T: Factory>() -> fn() -> T { T::make }
const fn create18<T: Factory>() -> fn() -> T { T::make }
const fn create19<T: Factory>() -> fn() -> T { T::make }
const fn create20<T: Factory>() -> fn() -> T { T::make }
const fn create21<T: Factory>() -> fn() -> T { T::make }
const fn create22<T: Factory>() -> fn() -> T { T::make }
const fn create23<T: Factory>() -> fn() -> T { T::make }
const fn create24<T: Factory>() -> fn() -> T { T::make }
const fn create25<T: Factory>() -> fn() -> T { T::make }
const fn create26<T: Factory>() -> fn() -> T { T::make }
const fn create27<T: Factory>() -> fn() -> T { T::make }
const fn create28<T: Factory>() -> fn() -> T { T::make }
const fn create29<T: Factory>() -> fn() -> T { T::make }
const fn create30<T: Factory>() -> fn() -> T { T::make }
const fn create31<T: Factory>() -> fn() -> T { T::make }
fn main(){}
