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

pub struct Interval{start:u8,end:u8}
const impl RangeBounds<u8> for Interval{
    fn start_bound(&self)->Bound<&u8>{Included(&self.start)}
    fn end_bound(&self)->Bound<&u8>{Excluded(&self.end)}
}
pub const CONTAINS:bool=<Interval as RangeBounds<u8>>::contains(&Interval{start:3,end:7},&4u8);
#[unsafe(no_mangle)]pub extern "C" fn range_entry()->bool{CONTAINS}
#[unsafe(no_mangle)]pub extern "C" fn range_probe(x:u8)->bool{<Interval as RangeBounds<u8>>::contains(&Interval{start:3,end:7},&x)}
