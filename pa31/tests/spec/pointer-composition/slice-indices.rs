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
