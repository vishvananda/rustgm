#![feature(const_trait_impl,const_cmp,const_range)]
use core::ops::Bound;use core::ops::Bound::{Included,Excluded,Unbounded};
pub const trait RangeBounds<T:?Sized>{fn start_bound(&self)->Bound<&T>;fn end_bound(&self)->Bound<&T>;
    fn contains<U>(&self, item: &U) -> bool
    where
        T: [const] PartialOrd<U>,
        U: ?Sized + [const] PartialOrd<T>,
    {
        (match self.start_bound() {
            Included(start) => start <= item,
            Excluded(start) => start < item,
            Unbounded => true,
        }) && (match self.end_bound() {
            Included(end) => item <= end,
            Excluded(end) => item < end,
            Unbounded => true,
        })
    }
}
fn main(){}
