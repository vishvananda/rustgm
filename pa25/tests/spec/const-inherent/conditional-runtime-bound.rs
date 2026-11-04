#![feature(const_trait_impl)]
const trait Give { fn get()->u8; } struct Token; const impl Give for Token { fn get()->u8{7} } struct Holder<T>(T); const impl<T:[const] Give> Holder<T> { fn read()->u8{T::get()} }
struct Ordinary;impl Give for Ordinary{fn get()->u8{9}}fn main(){let _=Holder::<Ordinary>::read();}
