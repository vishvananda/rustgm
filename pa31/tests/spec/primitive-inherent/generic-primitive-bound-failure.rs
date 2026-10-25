#![feature(rustc_attrs)]
#![rustc_coherence_is_core]
trait Marker{}impl<T:Marker>*const T{fn __rustgm_inherent(self)->u32{7}}fn failure(x:u32)->u32{1/x}
fn main(){if (0u64 as *const u8).__rustgm_inherent()!=7{let _=failure(0);}}
