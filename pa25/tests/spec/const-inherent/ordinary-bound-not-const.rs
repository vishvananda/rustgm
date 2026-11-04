#![feature(const_trait_impl)]
const trait Give { fn get()->u8; } struct Token; const impl Give for Token { fn get()->u8{7} } struct Holder<T>(T); const impl<T:Give> Holder<T> { fn read()->u8{T::get()} }
const BAD:u8=Holder::<Token>::read();fn main(){}
