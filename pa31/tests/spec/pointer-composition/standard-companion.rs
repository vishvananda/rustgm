#![feature(const_trait_impl,const_index,rustc_attrs,staged_api)]
#![unstable(feature="index_fixture",issue="none")]
#![allow(internal_features,dead_code)]
// Authored no_core foundation for the actual Index/IndexMut source protocol.
// Array/slice usize bodies retain real indexing, which becomes builtin places
// only after the ordinary trait, signature and implementation checks succeed.
#[rustc_const_unstable(feature="const_index",issue="143775")]
pub const trait Index<Idx:?Sized>{
    type Output:?Sized;
    fn index(&self,index:Idx)->&Self::Output;
}
#[rustc_const_unstable(feature="const_index",issue="143775")]
pub const trait IndexMut<Idx:?Sized>:[const]Index<Idx>{
    fn index_mut(&mut self,index:Idx)->&mut Self::Output;
}
#[rustc_const_unstable(feature="const_index",issue="143775")]
const impl<T,const N:usize> Index<usize> for [T;N]{
    type Output=T;
    fn index(&self,index:usize)->&T{&self[index]}
}
#[rustc_const_unstable(feature="const_index",issue="143775")]
const impl<T,const N:usize> IndexMut<usize> for [T;N]{
    fn index_mut(&mut self,index:usize)->&mut T{&mut self[index]}
}
// Actual slice impl companions require the Pointee/Metadata source foundation.
#[rustc_const_unstable(feature="const_index",issue="143775")]
const impl<T> Index<usize> for [T]{
    type Output=T;
    fn index(&self,index:usize)->&T{&self[index]}
}
#[rustc_const_unstable(feature="const_index",issue="143775")]
const impl<T> IndexMut<usize> for [T]{
    fn index_mut(&mut self,index:usize)->&mut T{&mut self[index]}
}
const READ:usize={let a=[7u8,8,9];let p=core::ptr::slice_from_raw_parts(&a as *const [u8;3] as *const u8,3);let s=unsafe{&*p};*<[u8] as Index<usize>>::index(s,2) as usize};
const WRITE:usize={let mut a=[7u8,8,9];let p=core::ptr::slice_from_raw_parts_mut(&mut a as *mut [u8;3] as *mut u8,3);let s=unsafe{&mut *p};*<[u8] as IndexMut<usize>>::index_mut(s,1)=91;*<[u8] as Index<usize>>::index(s,1) as usize};
#[unsafe(no_mangle)]pub extern "C" fn read()->usize{READ}
#[unsafe(no_mangle)]pub extern "C" fn write()->usize{WRITE}
