use core::mem::ManuallyDrop;
#[derive(Clone,Copy)]
union Generic<T,const N:usize=2> {values:ManuallyDrop<[T;N]>,word:u32}
#[derive(Clone,Copy)]
union Borrowed<'a,T:'a,const N:usize=2> where T:Clone {value:&'a T,values:ManuallyDrop<[T;N]>}
#[derive(Clone)]union Manual<const N:usize>{values:[u32;N]}
impl<const N:usize> Copy for Manual<N>{}
