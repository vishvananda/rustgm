#![feature(const_trait_impl,staged_api)]
#![unstable(feature="inherent_api",issue="none")]
const trait Give { fn get()->u8; } struct Token; const impl Give for Token { fn get()->u8{7} } struct Holder<T>(T); #[rustc_const_unstable(feature="inherent_const_api",issue="none")]const impl<T:[const] Give> Holder<T> { fn read()->u8{T::get()} }
struct Ordinary;impl Give for Ordinary{fn get()->u8{9}}const SEVEN:u8=Holder::<Token>::read();unsafe extern "C"{fn observe(v:u8);}fn main(){unsafe{observe(SEVEN);observe(Holder::<Ordinary>::read());}}
