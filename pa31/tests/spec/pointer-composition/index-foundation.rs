// Authored no_core foundation for the actual Index/IndexMut source protocol.
// Array/slice usize bodies retain real indexing, which becomes builtin places
// only after the ordinary trait, signature and implementation checks succeed.
#[rustc_const_unstable(feature="const_index",issue="143775")]
#[lang="index"]
pub const trait Index<Idx:?Sized>{
    #[rustc_diagnostic_item="IndexOutput"]
    type Output:?Sized;
    fn index(&self,index:Idx)->&Self::Output;
}
#[rustc_const_unstable(feature="const_index",issue="143775")]
#[lang="index_mut"]
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
