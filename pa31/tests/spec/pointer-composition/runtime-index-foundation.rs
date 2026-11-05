// Actual const trait protocol with ordinary runtime array implementations.
// Array/slice usize bodies retain real indexing, which becomes builtin places
// only after the ordinary trait, signature and implementation checks succeed.
#[lang="index"]
pub const trait Index<Idx:?Sized>{
    #[rustc_diagnostic_item="IndexOutput"]
    type Output:?Sized;
    fn index(&self,index:Idx)->&Self::Output;
}
#[lang="index_mut"]
pub const trait IndexMut<Idx:?Sized>:[const]Index<Idx>{
    fn index_mut(&mut self,index:Idx)->&mut Self::Output;
}
impl<T,const N:usize> Index<usize> for [T;N]{
    type Output=T;
    fn index(&self,index:usize)->&T{&self[index]}
}
impl<T,const N:usize> IndexMut<usize> for [T;N]{
    fn index_mut(&mut self,index:usize)->&mut T{&mut self[index]}
}
