#![feature(const_trait_impl,const_destruct,const_cmp)]
use core::marker::Destruct;
use core::cmp::Ord;
pub const fn min_by_key<T, F, K>(v1: T, v2: T, mut f: F) -> T
where
    T: [const] Destruct,
    F: [const] FnMut(&T) -> K + [const] Destruct,
    K: [const] Ord + [const] Destruct,
{
    if f(&v2) < f(&v1) { v2 } else { v1 }
}
fn main(){}
